#[macro_use]
extern crate log;

use humantime::format_duration;
use oxigraph::io::GraphFormat;
use oxigraph::model::Quad;
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

    let base_path = path::PathBuf::new().join("src/data");
    let merged_ontologies_path = base_path.clone().join("merged-ontologies.nt");
    let mut merged_ontologies_graph = cam_pipeline_rust::deserialize_graph(&merged_ontologies_path)?;

    let store = cam_pipeline_rust::get_store(vec![merged_ontologies_graph])?;
    let results = store.query(include_str!("../sparql/subclass-closure.rq"))?;

    let output_path: path::PathBuf = base_path.clone().join("subclass-closure.nt");
    let output_file = fs::File::create(&output_path)?;
    let mut writer = io::BufWriter::new(output_file);
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
