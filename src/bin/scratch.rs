#[macro_use]
extern crate log;

use horned_owl::ontology;
use horned_owl::ontology::indexed::ThreeIndexedOntology;
use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::graph::MutableGraph;
use sophia::ns;
use sophia::parser;
use sophia::serializer::TripleSerializer;
use sophia::term;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    // let cam_pipeline_path = path::PathBuf::new().join("/home/jdr0887/workspace/github/NCATS-Tangerine/cam-pipeline-rust");
    // let merged_ontologies_path = cam_pipeline_path.clone().join("merged-ontologies.nt");
    // let merged_ontologies_graph = cam_pipeline_rust::deserialize_graph(&merged_ontologies_path)?;
    //
    // debug!("starting reasoner");
    // let mut reasoner = reasonable::owl::Reasoner::new();
    //
    // let triples: Vec<(&str, &str, &str)> = merged_ontologies_graph
    //     .triples()
    //     .map_triples(|t| (string_to_str(t.s().value().to_string()), string_to_str(t.p().value().to_string()), string_to_str(t.o().value().to_string())))
    //     .into_iter()
    //     .collect::<Result<Vec<(&str, &str, &str)>, _>>()?;
    //
    // reasoner.load_triples_str(triples);
    //
    // // reasoner.load_file(&merged_ontologies_path.to_string_lossy());
    // reasoner.reason();
    //
    // debug!("finished reasoner");

    let re = regex::Regex::new(r"^[^<https://w3id.org/biolink/vocab/logical_interpretation_enum>].+<https://w3id.org/biolink/vocab/(.+:.+)>")?;
    let line = "    skos:exactMatch <https://w3id.org/biolink/vocab/SIO:000984>,";
    debug!("matches: {}", re.is_match(line));
    debug!("capture: {:?}", &re.captures(line).unwrap()[1].replace(":", "_"));

    let line = "<https://w3id.org/biolink/vocab/logical_interpretation_enum> skos:inScheme <https://w3id.org/biolink/biolink-model> ;";
    debug!("matches: {}", re.is_match(line));
    debug!("capture: {:?}", &re.captures(line).unwrap()[1].replace(":", "_"));

    // cam_pipeline_rust::get_biolink_model(&path::PathBuf::from("/tmp/biolink.ttl"))?;

    // let swrl_urn_ns = ns::Namespace::new("urn:swrl#")?;
    // let swrl_urn_x = swrl_urn_ns.get("x")?;
    //
    // let swrl_ns = ns::Namespace::new("http://www.w3.org/2003/11/swrl#")?;
    //
    // let swrl_imp = swrl_ns.get("Imp")?;
    // let swrl_variable = swrl_ns.get("Variable")?;
    // let swrl_atom_list = swrl_ns.get("AtomList")?;
    // let swrl_class_atom = swrl_ns.get("ClassAtom")?;
    // let swrl_class_predicate = swrl_ns.get("classPredicate")?;
    // let swrl_argument_1 = swrl_ns.get("argument1")?;
    // let swrl_head = swrl_ns.get("head")?;
    // let swrl_body = swrl_ns.get("body")?;
    //
    // let so_gene = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/SO_0000704")?;
    //
    // let mut output_graph: FastGraph = FastGraph::new();
    // // output_graph.insert(&swrl_urn_x, &ns::rdf::type_, &swrl_variable).unwrap();
    //
    // let obj_ns = ns::Namespace::new("http://identifiers.org/uniprot/")?;
    // let obj = obj_ns.get("Q6XZB0")?;
    //
    // let sub_ns = ns::Namespace::new("http://purl.obolibrary.org/obo/go/extensions/reacto.owl#")?;
    // let sub = sub_ns.get("REACTO_R-HSA-6792451")?;
    //
    // let mut count: i32 = 4000001;
    // output_graph.insert(&obj, &ns::rdf::type_, &ns::owl::Class).unwrap();
    // output_graph.insert(&obj, &ns::rdfs::subClassOf, &so_gene).unwrap();
    //
    // let id_2 = format!("riog{:08}", count).to_string();
    // let sub_bn_2: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_2).unwrap();
    // count += 1;
    //
    // let id_3 = format!("riog{:08}", count).to_string();
    // let sub_bn_3: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_3).unwrap();
    // count += 1;
    //
    // let id_4 = format!("riog{:08}", count).to_string();
    // let sub_bn_4: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_4).unwrap();
    // count += 1;
    //
    // let id_5 = format!("riog{:08}", count).to_string();
    // let sub_bn_5: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_5).unwrap();
    // count += 1;
    //
    // let id_6 = format!("riog{:08}", count).to_string();
    // let sub_bn_6: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_6).unwrap();
    // count += 1;
    //
    // output_graph.insert(&sub_bn_2, &ns::rdf::type_, &swrl_imp).unwrap();
    // output_graph.insert(&sub_bn_2, &swrl_body, &sub_bn_3).unwrap();
    // output_graph.insert(&sub_bn_3, &ns::rdf::type_, &swrl_atom_list).unwrap();
    // output_graph.insert(&sub_bn_3, &ns::rdf::first, &sub_bn_4).unwrap();
    // output_graph.insert(&sub_bn_4, &ns::rdf::type_, &swrl_class_atom).unwrap();
    // output_graph.insert(&sub_bn_4, &swrl_class_predicate, &sub).unwrap();
    // output_graph.insert(&sub_bn_4, &swrl_argument_1, &swrl_urn_x).unwrap();
    // output_graph.insert(&sub_bn_3, &ns::rdf::rest, &ns::rdf::nil).unwrap();
    // output_graph.insert(&sub_bn_2, &swrl_head, &sub_bn_5).unwrap();
    // output_graph.insert(&sub_bn_5, &ns::rdf::type_, &swrl_atom_list).unwrap();
    // output_graph.insert(&sub_bn_5, &ns::rdf::first, &sub_bn_6).unwrap();
    // output_graph.insert(&sub_bn_6, &ns::rdf::type_, &swrl_class_atom).unwrap();
    // output_graph.insert(&sub_bn_6, &swrl_class_predicate, &obj).unwrap();
    // output_graph.insert(&sub_bn_6, &swrl_argument_1, &swrl_urn_x).unwrap();
    // output_graph.insert(&sub_bn_5, &ns::rdf::rest, &ns::rdf::nil).unwrap();
    //
    // let base_path: path::PathBuf = path::PathBuf::new().join("/home/jdr0887/workspace/github/NCATS-Tangerine");
    // let cam_pipeline_path: path::PathBuf = base_path.clone().join("cam-pipeline");
    // let output: path::PathBuf = cam_pipeline_path.clone().join("reacto-uniprot-rules.rust.cmp.nt");
    // cam_pipeline_rust::serialize_graph(&output, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn string_to_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
