#[macro_use]
extern crate log;

use std::error;
use std::path;
use std::time::Instant;

use humantime::format_duration;
use rayon::prelude::*;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use structopt::StructOpt;
use walkdir::DirEntry;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    // let base_path: path::PathBuf = path::PathBuf::new().join("src/data");
    let base_path: path::PathBuf = path::PathBuf::new().join("/tmp/owx");
    let tmp_path: path::PathBuf = base_path.clone().join("tmp");
    let ttl_files: Vec<DirEntry> =
        walkdir::WalkDir::new(&tmp_path).min_depth(1).max_depth(1).into_iter().filter_map(|x| x.ok()).filter(|x| x.file_name().to_string_lossy().ends_with(".ttl")).collect();

    let mut graph = FastGraph::new();

    for ont in ttl_files.iter() {
        let tmp_graph = cam_pipeline_rust::deserialize_graph(&ont.path().to_path_buf())?;
        tmp_graph.triples().add_to_graph(&mut graph)?;
    }

    let output_path: path::PathBuf = base_path.clone().join("merged-ctd-ontology.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
