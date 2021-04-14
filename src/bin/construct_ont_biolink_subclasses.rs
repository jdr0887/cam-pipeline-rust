#[macro_use]
extern crate log;

use humantime::format_duration;
use oxigraph::io::GraphFormat;
use oxigraph::model::NamedOrBlankNode;
use oxigraph::model::Quad;
use oxigraph::model::*;
use oxigraph::sparql::{QueryResults, QueryResultsFormat};
use oxigraph::MemoryStore;
use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::ns;
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

    let base_path: path::PathBuf = path::PathBuf::new().join("src/data");

    let biolink_model_path: path::PathBuf = base_path.clone().join("biolink-model.ttl");
    let mut biolink_model_graph = cam_pipeline_rust::get_biolink_model(&biolink_model_path)?;

    let biolink_local_path: path::PathBuf = base_path.clone().join("biolink-local.ttl");
    let mut biolink_local_graph = cam_pipeline_rust::deserialize_graph(&biolink_local_path)?;

    let output_path: path::PathBuf = base_path.clone().join("ont-biolink-subclasses.nt");
    let output_file = fs::File::create(&output_path)?;
    let mut writer = io::BufWriter::new(output_file);
    let store = cam_pipeline_rust::get_store(vec![biolink_model_graph, biolink_local_graph])?;
    let results = store.query(include_str!("../sparql/construct-ont-biolink-subclasses.rq"))?;
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
