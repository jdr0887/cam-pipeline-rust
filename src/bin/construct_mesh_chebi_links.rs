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
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "construct_mesh_chebi_links", about = "construct mesh CHEBI links")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,

    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let noctua_ontology_graph = cam_pipeline_rust::deserialize_graph(&options.input)?;
    let output_graph = mesh_chebi_links(&noctua_ontology_graph)?;
    cam_pipeline_rust::serialize_graph(&options.output, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

// PREFIX ChemicalEntity: <http://purl.obolibrary.org/obo/CHEBI_24431>
// CONSTRUCT {
// ?term rdfs:subClassOf ChemicalEntity: .
// }
// WHERE {
// ?s rdf:type ?term .
// # This will not be valid if other MeSH terms appear in the database
// FILTER (STRSTARTS(STR(?term), "http://id.nlm.nih.gov/mesh"))
// FILTER (isIRI(?term))
// }
fn mesh_chebi_links(noctua_ontology_graph: &FastGraph) -> Result<FastGraph, Box<dyn error::Error>> {
    let chemical_entity = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/CHEBI_24431")?;

    let mut output_graph = FastGraph::new();
    noctua_ontology_graph.triples_with_p(&ns::rdf::type_).filter_triples(|t| t.o().value().to_string().starts_with("http://identifiers.org/mesh")).for_each_triple(|t| {
        output_graph.insert(t.o(), &ns::rdfs::subClassOf, &chemical_entity).unwrap();
    })?;

    Ok(output_graph)
}
