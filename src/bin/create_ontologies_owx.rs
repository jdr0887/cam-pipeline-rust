#[macro_use]
extern crate log;

use horned_owl::io::owx;
use horned_owl::model::*;
use horned_owl::ontology;
use horned_owl::vocab::WithIRI;
use sophia::term::Term;
use sophia_api::term::TTerm;
use sophia_api::triple::stream::TripleSource;
use std::fs;
use std::io;
use std::path;
use std::rc::Rc;

type SpTerm = Term<Rc<str>>;

fn main() -> io::Result<()> {
    env_logger::init();

    let base_path: path::PathBuf = path::PathBuf::new().join("src/data");
    let ontologies_owx_path: path::PathBuf = base_path.clone().join("ontologies.owx");

    let mut prefix_mapping = curie::PrefixMapping::default();
    prefix_mapping.add_prefix("owl", "http://www.w3.org/2002/07/owl#").unwrap();
    prefix_mapping.add_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#").unwrap();
    prefix_mapping.add_prefix("xml", "http://www.w3.org/XML/1998/namespace").unwrap();
    prefix_mapping.add_prefix("xsd", "http://www.w3.org/2001/XMLSchema#").unwrap();
    prefix_mapping.add_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#").unwrap();

    let build = Build::new();
    // let mut ontology = ontology::axiom_mapped::AxiomMappedOntology::default();
    let mut ontology = ontology::set::SetOntology::default();
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/uberon/uberon-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/pato.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/cl/cl-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/bspo.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/go/go-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/go/imports/chebi_import.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/hp/hp-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/mondo/mondo-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/chebi.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/ro/ro-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/eco/eco-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/uberon/bridge/uberon-bridge-to-caro.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/uberon/bridge/cl-bridge-to-caro.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/go/extensions/go-bfo-bridge.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/ncbitaxon/subsets/taxslim.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/ncbitaxon/subsets/taxslim-disjoint-over-in-taxon.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/wbbt.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/wbphenotype/wbphenotype-base.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/wbphenotype/imports/wbls_import.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/uberon/bridge/uberon-bridge-to-wbbt.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/uberon/bridge/cl-bridge-to-wbbt.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/ddanat.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/zfa.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/emapa.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/go/noctua/neo.owl")));
    ontology.insert(Import(build.iri("http://purl.obolibrary.org/obo/go/snapshot/extensions/reacto.owl")));

    // ontology.declare(build.class("http://purl.obolibrary.org/obo/IAO_0000311>"));
    ontology.declare(build.annotation_property("http://purl.obolibrary.org/obo/IAO_0000136"));
    // ontology.declare(build.object_property("http://purl.obolibrary.org/obo/SEPIO_0000124"));

    // AnnotationAssertion(rdfs:label <http://purl.obolibrary.org/obo/IAO_0000136> "is about"@en)

    let aa = AnnotationAssertion {
        subject: build.iri("http://purl.obolibrary.org/obo/IAO_0000136"),
        ann: Annotation {
            ap: build.annotation_property(horned_owl::vocab::RDFS::Label.iri_s()),
            //ap: build.annotation_property("rdfs:label"),
            av: AnnotationValue::Literal(Literal::Language { literal: "is about".into(), lang: "en".into() }),
        },
    };

    ontology.insert(aa);

    // ontology.insert(AnnotationAssertion::new(
    //     build.iri("http://purl.obolibrary.org/obo/SEPIO_0000124"),
    //     Annotation {
    //         ap: build.annotation_property("rdfs:label").into(),
    //         av: AnnotationValue::Literal(Literal::Language {
    //             literal: "has supporting reference".into(),
    //             lang: "en".into(),
    //         }),
    //     },
    // ));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/DDANAT_0000401")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CL_0000003")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_44689"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/DDANAT_0010001")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CARO_0000006")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_44689"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(SubClassOf {
    //     sup: ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/EMAPA_0")),
    //     sub: ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CARO_0000000")),
    // });

    // ontology.insert(SubClassOf {
    //     sup: ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/EMAPA_0")),
    //     sub: ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CARO_0000000")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_10090"),
    //             )),
    //         },
    //     ]),
    // });

    // ontology.insert(AnnotationAssertion::new(build.iri("http://purl.obolibrary.org/obo/IAO_0000311"),
    //     Annotation {
    //         ap: build.annotation_property("rdfs:label").into(),
    //         av: AnnotationValue::Literal(Literal::Language {
    //             literal: "publication".into(),
    //             lang: "en".into(),
    //         }),
    //     },
    // ));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/PO_0025131")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CARO_0000000")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_33090"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/PO_0025606")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CL_0000003")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_33090"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/ZFA_0009000")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CL_0000003")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_7955"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(EquivalentClasses(vec![
    //     ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/ZFA_0100000")),
    //     ClassExpression::ObjectIntersectionOf(vec![
    //         ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/CARO_0000000")),
    //         ClassExpression::ObjectSomeValuesFrom {
    //             ope: ObjectPropertyExpression::from(
    //                 build.object_property("http://purl.obolibrary.org/obo/RO_0002162"),
    //             ),
    //             bce: Box::new(ClassExpression::Class(
    //                 build.class("http://purl.obolibrary.org/obo/NCBITaxon_7955"),
    //             )),
    //         },
    //     ]),
    // ]));

    // ontology.insert(SubClassOf {
    //     sup: ClassExpression::ObjectSomeValuesFrom {
    //         ope: ObjectPropertyExpression::from(
    //             build.object_property("http://purl.obolibrary.org/obo/emapa#ends_at"),
    //         ),
    //         bce: Box::new(ClassExpression::Class(
    //             build.class("http://purl.obolibrary.org/obo/TS_0"),
    //         )),
    //     },
    //     sub: ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/EMAPA_0")),
    // });

    // ontology.insert(SubClassOf {
    //     sup: ClassExpression::ObjectSomeValuesFrom {
    //         ope: ObjectPropertyExpression::from(
    //             build.object_property("http://purl.obolibrary.org/obo/emapa#starts_at"),
    //         ),
    //         bce: Box::new(ClassExpression::Class(
    //             build.class("http://purl.obolibrary.org/obo/TS_0"),
    //         )),
    //     },
    //     sub: ClassExpression::Class(build.class("http://purl.obolibrary.org/obo/EMAPA_0")),
    // });

    // let summary = command::summary::summarize(ontology);
    // info!("{:?}", summary);
    let file = fs::File::create(&ontologies_owx_path).ok().unwrap();
    owx::writer::write(&mut io::BufWriter::new(file), &ontology.into(), Some(&prefix_mapping)).ok().unwrap();

    let file = fs::File::open(&ontologies_owx_path)?;
    let bufreader = io::BufReader::new(file);
    let triple_iter = sophia::parser::xml::parse_bufread(bufreader);
    let triple_result: Result<Vec<_>, _> = triple_iter.collect_triples();
    let triple_v: Vec<[SpTerm; 3]> = triple_result.unwrap();
    for triple in triple_v {
        info!("{}\n\t{}\n\t{}", &triple[0].value(), &triple[1].value(), &triple[2].value());
    }

    Ok(())
}
