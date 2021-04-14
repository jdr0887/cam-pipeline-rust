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

    let noctua_ontology_path: path::PathBuf = base_path.clone().join("noctua-reactome-ontology.nt");
    let noctua_ontology_graph = cam_pipeline_rust::deserialize_graph(&noctua_ontology_path)?;

    let output_graph = protein_subclasses(&noctua_ontology_graph)?;

    let output_path: path::PathBuf = base_path.clone().join("protein-subclasses.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

// PREFIX pro: <http://purl.obolibrary.org/obo/PR_000000001>
// PREFIX protein: <http://purl.obolibrary.org/obo/CHEBI_36080>
// CONSTRUCT {
// ?term rdfs:subClassOf pro: .
// ?term rdfs:subClassOf protein: .
// }
// WHERE {
// ?s ?p ?term .
// FILTER (STRSTARTS(STR(?term), "http://identifiers.org/uniprot"))
// FILTER (isIRI(?term))
// }
fn protein_subclasses(noctua_ontology_graph: &FastGraph) -> Result<FastGraph, Box<dyn error::Error>> {
    let mut output_graph = FastGraph::new();

    let pro = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/PR_000000001")?;
    let protein = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/CHEBI_36080")?;

    noctua_ontology_graph.triples().filter_triples(|t| t.o().value().to_string().starts_with("http://identifiers.org/uniprot")).for_each_triple(|t| {
        output_graph.insert(t.o(), &ns::rdfs::subClassOf, &pro).unwrap();
        output_graph.insert(t.o(), &ns::rdfs::subClassOf, &protein).unwrap();
    })?;

    Ok(output_graph)
}
