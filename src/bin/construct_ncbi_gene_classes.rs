#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::error;
use std::path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let base_path: path::PathBuf = path::PathBuf::new().join("src/data");
    let uniprot_to_ncbi_rules_path: path::PathBuf = base_path.clone().join("uniprot-to-ncbi-rules.ttl");
    let uniprot_to_ncbi_rules_graph = cam_pipeline_rust::deserialize_graph(&uniprot_to_ncbi_rules_path)?;

    let output_graph = ncbi_gene_classes(&uniprot_to_ncbi_rules_graph)?;

    let output_path: path::PathBuf = base_path.clone().join("ncbi-gene-classes.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

// PREFIX Gene: <http://purl.obolibrary.org/obo/SO_0000704>
// CONSTRUCT {
// ?term rdfs:subClassOf Gene: .
// }
// WHERE {
// ?s ?p ?term .
// FILTER (STRSTARTS(STR(?term), "http://identifiers.org/ncbigene"))
// FILTER (isIRI(?term))
// }
fn ncbi_gene_classes(input_graph: &FastGraph) -> Result<FastGraph, Box<dyn error::Error>> {
    let gene = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/SO_0000704")?;
    let mut output_graph = FastGraph::new();
    input_graph.triples().filter_triples(|t| t.o().value().to_string().starts_with("http://identifiers.org/ncbigene")).for_each_triple(|t| {
        output_graph.insert(t.o(), &ns::rdfs::subClassOf, &gene).unwrap();
    })?;
    Ok(output_graph)
}
