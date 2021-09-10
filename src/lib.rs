extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate sophia;

use oxigraph::model::Quad;
use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::graph::MutableGraph;
use sophia::ns;
use sophia::serializer::TripleSerializer;
use sophia::term;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::collections::HashMap;
use std::error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path;

lazy_static! {
    pub static ref SOPHIA_TO_OXIGRAPH_MAP: HashMap<sophia::term::SimpleIri<'static>, oxigraph::model::NamedNodeRef<'static>> = {
        let mut map: HashMap<sophia::term::SimpleIri<'static>, oxigraph::model::NamedNodeRef<'static>> = HashMap::new();
        map.insert(ns::xsd::string, oxigraph::model::vocab::xsd::STRING);
        map.insert(ns::xsd::decimal, oxigraph::model::vocab::xsd::DECIMAL);
        map.insert(ns::xsd::double, oxigraph::model::vocab::xsd::DOUBLE);
        map.insert(ns::xsd::integer, oxigraph::model::vocab::xsd::INTEGER);
        map.insert(ns::xsd::int, oxigraph::model::vocab::xsd::INT);
        map.insert(ns::xsd::float, oxigraph::model::vocab::xsd::FLOAT);
        map.insert(ns::xsd::anyURI, oxigraph::model::vocab::xsd::ANY_URI);
        map.insert(ns::xsd::boolean, oxigraph::model::vocab::xsd::BOOLEAN);
        map.insert(ns::xsd::dateTime, oxigraph::model::vocab::xsd::DATE_TIME);
        map.insert(ns::xsd::date, oxigraph::model::vocab::xsd::DATE);
        map.insert(ns::xsd::long, oxigraph::model::vocab::xsd::LONG);
        map.insert(ns::xsd::nonNegativeInteger, oxigraph::model::vocab::xsd::NON_NEGATIVE_INTEGER);
        map.insert(ns::xsd::short, oxigraph::model::vocab::xsd::SHORT);
        map
    };
}

pub fn serialize_graph(output_path: &path::PathBuf, graph: &FastGraph) -> Result<(), Box<dyn error::Error>> {
    let output = fs::File::create(&output_path).expect(format!("can't create {}", output_path.to_string_lossy()).as_str());
    info!("writing: {}", output_path.to_string_lossy());
    let writer = io::BufWriter::new(output);
    let mut serializer = sophia::serializer::nt::NtSerializer::new(writer);
    serializer.serialize_graph(graph)?;
    Ok(())
}

pub fn deserialize_graph(input_path: &path::PathBuf) -> Result<FastGraph, Box<dyn error::Error>> {
    let input = fs::File::open(&input_path).expect(format!("can't open {}", input_path.to_string_lossy()).as_str());
    info!("reading: {}", input_path.to_string_lossy());
    let reader = io::BufReader::new(input);
    let graph: FastGraph = match input_path.extension().and_then(OsStr::to_str) {
        Some("ttl") => sophia::parser::turtle::parse_bufread(reader).collect_triples().unwrap(),
        Some("nt") => sophia::parser::nt::parse_bufread(reader).collect_triples().unwrap(),
        Some("xml") | Some("rdf") | Some("owl") => sophia::parser::xml::parse_bufread(reader).collect_triples().unwrap(),
        _ => panic!("invalid extension"),
    };
    Ok(graph)
}

pub fn insert_terms_into_graph(graph: &mut FastGraph, files: &Vec<path::PathBuf>, check_for_production_modelstate: bool) -> Result<(), Box<dyn error::Error>> {
    let production_term = term::StaticTerm::new_literal_dt("production", ns::xsd::string)?;
    let s = ns::Namespace::new("http://geneontology.org/lego/")?;
    let modelstate = s.get("modelstate")?;
    for file in files.iter() {
        let tmp_graph = deserialize_graph(&file)?;

        if check_for_production_modelstate {
            let good_graph = tmp_graph.triples_with_po(&modelstate, &production_term).next().is_some();
            if good_graph {
                for triple in tmp_graph.triples() {
                    let triple = triple?;
                    graph.insert(triple.s(), triple.p(), triple.o())?;
                }
                debug!("tmp_graph.triples().count(): {}, graph.triples().count(): {}", tmp_graph.triples().count(), graph.triples().count());
            } else {
                debug!("skipping due to not production modelstate: {:?}", file);
            }
        } else {
            for triple in tmp_graph.triples() {
                let triple = triple?;
                graph.insert(triple.s(), triple.p(), triple.o())?;
            }
            debug!("graph.triples().count(): {}, graph.triples().count(): {}", tmp_graph.triples().count(), graph.triples().count());
        }
    }
    Ok(())
}

pub fn load_graphs_into_memory_store(graphs: Vec<FastGraph>) -> Result<oxigraph::MemoryStore, Box<dyn error::Error>> {
    debug!("getting store");
    let store = oxigraph::MemoryStore::new();

    for graph in graphs.iter() {
        graph.triples().for_each_triple(|t| {
            // debug!("t.s(): {:?}, t.p(): {:?}, t.o(): {:?}", t.s(), t.p(), t.o());

            let subject = {
                match t.s().kind() {
                    term::TermKind::Iri => oxigraph::model::NamedOrBlankNode::NamedNode(oxigraph::model::NamedNode::new(t.s().value()).unwrap()),
                    term::TermKind::BlankNode => oxigraph::model::NamedOrBlankNode::BlankNode(oxigraph::model::BlankNode::new(t.s().value_raw().0).unwrap()),
                    _ => return (),
                }
            };

            let predicate = {
                match t.p().kind() {
                    term::TermKind::Iri => oxigraph::model::NamedNode::new(t.p().value()).unwrap(),
                    _ => return (),
                }
            };

            let object = {
                match t.o().kind() {
                    term::TermKind::Iri => oxigraph::model::Term::NamedNode(oxigraph::model::NamedNode::new(t.o().value()).unwrap()),
                    term::TermKind::BlankNode => oxigraph::model::Term::BlankNode(oxigraph::model::BlankNode::new(t.o().value_raw().0).unwrap()),
                    term::TermKind::Literal => match t.o().language() {
                        Some(tag) => oxigraph::model::Term::Literal(oxigraph::model::Literal::new_language_tagged_literal(t.o().value_raw().0, tag).unwrap()),
                        None => {
                            let datatype = t.o().datatype().unwrap();
                            let x = if SOPHIA_TO_OXIGRAPH_MAP.contains_key(&datatype) {
                                oxigraph::model::Term::Literal(oxigraph::model::Literal::new_typed_literal(
                                    t.o().value_raw().0,
                                    SOPHIA_TO_OXIGRAPH_MAP.get(&datatype).unwrap().into_owned(),
                                ))
                            } else {
                                debug!("datatype not in cached map: {:?}", datatype);
                                oxigraph::model::Term::Literal(oxigraph::model::Literal::new_simple_literal(t.o().value_raw().0))
                            };
                            x
                        }
                    },
                    _ => return (),
                }
            };

            let quad = Quad::new(subject, predicate, object, None);
            store.insert(quad);
        })?;
    }

    Ok(store)
}

pub fn load_graphs_into_rocksdb_store(db_path: &path::PathBuf, graphs: Vec<FastGraph>) -> Result<oxigraph::RocksDbStore, Box<dyn error::Error>> {
    debug!("getting store");
    let store = oxigraph::RocksDbStore::open(db_path)?;

    for graph in graphs.iter() {
        graph.triples().for_each_triple(|t| {
            // debug!("t.s(): {:?}, t.p(): {:?}, t.o(): {:?}", t.s(), t.p(), t.o());

            let subject = {
                match t.s().kind() {
                    term::TermKind::Iri => oxigraph::model::NamedOrBlankNode::NamedNode(oxigraph::model::NamedNode::new(t.s().value()).unwrap()),
                    term::TermKind::BlankNode => oxigraph::model::NamedOrBlankNode::BlankNode(oxigraph::model::BlankNode::new(t.s().value_raw().0).unwrap()),
                    _ => return (),
                }
            };

            let predicate = {
                match t.p().kind() {
                    term::TermKind::Iri => oxigraph::model::NamedNode::new(t.p().value()).unwrap(),
                    _ => return (),
                }
            };

            let object = {
                match t.o().kind() {
                    term::TermKind::Iri => oxigraph::model::Term::NamedNode(oxigraph::model::NamedNode::new(t.o().value()).unwrap()),
                    term::TermKind::BlankNode => oxigraph::model::Term::BlankNode(oxigraph::model::BlankNode::new(t.o().value_raw().0).unwrap()),
                    term::TermKind::Literal => match t.o().language() {
                        Some(tag) => oxigraph::model::Term::Literal(oxigraph::model::Literal::new_language_tagged_literal(t.o().value_raw().0, tag).unwrap()),
                        None => {
                            let datatype = t.o().datatype().unwrap();
                            let x = if SOPHIA_TO_OXIGRAPH_MAP.contains_key(&datatype) {
                                oxigraph::model::Term::Literal(oxigraph::model::Literal::new_typed_literal(
                                    t.o().value_raw().0,
                                    SOPHIA_TO_OXIGRAPH_MAP.get(&datatype).unwrap().into_owned(),
                                ))
                            } else {
                                debug!("datatype not in cached map: {:?}", datatype);
                                oxigraph::model::Term::Literal(oxigraph::model::Literal::new_simple_literal(t.o().value_raw().0))
                            };
                            x
                        }
                    },
                    _ => return (),
                }
            };

            let quad = Quad::new(subject, predicate, object, None);
            store.insert(&quad).unwrap();
        })?;
    }

    Ok(store)
}

#[cfg(test)]
mod test {

    #[test]
    fn test_regex_replacement_for_is_defined_by() {
        let data = vec![
            ("http://purl.obolibrary.org/obo/CHEBI_74380", "http://purl.obolibrary.org/obo/chebi.owl"),
            ("http://purl.obolibrary.org/obo/RefSeq#_NM_001025599", "http://purl.obolibrary.org/obo/refseq#_nm.owl"),
            ("http://purl.obolibrary.org/obo/RNAcentral#_URS00004DCB4C_9606", "http://purl.obolibrary.org/obo/rnacentral#_urs00004dcb4c.owl"),
        ];

        let re = regex::Regex::new(r"http://purl.obolibrary.org/obo/(.+_.+)").unwrap();

        data.iter().for_each(|(item, expectation)| {
            assert_eq!(re.is_match(&item), true);
            let capture = re.captures_iter(&item).next().unwrap();
            let suffix = capture.get(1).unwrap().as_str();
            let idx = suffix.rfind("_").unwrap();
            let trimmed_suffix = suffix.split_at(idx).0;
            let mut value = "http://purl.obolibrary.org/obo/".to_string();
            value.push_str(trimmed_suffix);
            value.push_str(".owl");
            assert_eq!(expectation, &value.to_lowercase().as_str());
        });
    }
}
