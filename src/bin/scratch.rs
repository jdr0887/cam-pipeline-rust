#[macro_use]
extern crate log;

use horned_owl::ontology;
use horned_owl::ontology::indexed::ThreeIndexedOntology;
use humantime::format_duration;
use itertools::Itertools;
use rayon::prelude::*;
use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::graph::MutableGraph;
use sophia::ns;
use sophia::parser;
use sophia::serializer::TripleSerializer;
use sophia::term;
use sophia::term::{SimpleIri, TTerm, TermKind};
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let cam_reasoned_path = path::PathBuf::new().join("/home/jdr0887/cam-db-reasoned.ttl");
    let cam_reasoned_graph = cam_pipeline_rust::deserialize_graph(&cam_reasoned_path)?;

    // let biolink_ns = ns::Namespace::new("https://w3id.org/biolink/vocab/")?;
    // let biolink_pathway = biolink_ns.get("Pathway")?;
    // let biolink_gene_or_gene_product = biolink_ns.get("GeneOrGeneProduct")?;
    //
    // cam_reasoned_graph.triples_with_po(&ns::rdfs::subClassOf, &biolink_pathway).for_each_triple(|t| {
    //     let obj = t.o().value().to_string();
    //     let sub = t.s().value().to_string();
    //     debug!("sub: {:?}, obj: {:?}", sub, obj);
    // })?;

    let ncbigene_ns = ns::Namespace::new("http://identifiers.org/ncbigene/")?;
    let gene = ncbigene_ns.get("6658")?;

    cam_reasoned_graph.triples().filter_triples(|t| t.s().value().to_string() == gene.to_string() || t.o().value().to_string() == gene.to_string()).for_each_triple(|t| {
        let obj = t.o().value().to_string();
        let pred = t.p().value().to_string();
        let sub = t.s().value().to_string();
        debug!("sub: {:?}, pred: {:?} obj: {:?}", sub, pred, obj);
    })?;

    // let pubchem_compound_ns = ns::Namespace::new("http://identifiers.org/pubchem.compound/")?;
    // let pubchem_compound = pubchem_compound_ns.get("5291")?;
    // let pubchem_compound = pubchem_compound_ns.get("2662")?;

    // cam_reasoned_graph
    //     .triples()
    //     .filter_triples(|t| t.s().value().to_string() == pubchem_compound.to_string() || t.o().value().to_string() == pubchem_compound.to_string())
    //     .for_each_triple(|t| {
    //         let obj = t.o().value().to_string();
    //         let pred = t.p().value().to_string();
    //         let sub = t.s().value().to_string();
    //         debug!("sub: {:?}, pred: {:?} obj: {:?}", sub, pred, obj);
    //     })?;

    // let output_path: path::PathBuf = path::PathBuf::new().join("/home/jdr0887/results.txt");
    // let mut writer = io::BufWriter::new(fs::File::create(output_path.as_path())?);
    // for (key, value) in results.iter() {
    //     for v in value.iter() {
    //         writer.write_all(format!("{},id_prefixes,{}\n", key, v).as_bytes()).expect("Unable to write data");
    //     }
    // }

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
