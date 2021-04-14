#[macro_use]
extern crate log;

use humantime::format_duration;
use itertools::Itertools;
use oxigraph::io::GraphFormat;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term;
use sophia::term::{SimpleIri, TTerm};
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
    cam_pipeline_rust::get_biolink_model(&biolink_model_path)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
