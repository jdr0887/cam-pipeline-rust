#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use std::error;
use std::fs;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create_noctua_reactome_ontology", about = "create noctua reactome ontology")]
struct Options {
    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let noctua_models_path: path::PathBuf = path::PathBuf::new().join("noctua-models/models");

    let output_graph = noctua_reactome_ontology(&noctua_models_path)?;

    let output_path: path::PathBuf = options.output;
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn noctua_reactome_ontology(noctua_models_path: &path::PathBuf) -> Result<FastGraph, Box<dyn error::Error>> {
    let mut output_graph = FastGraph::new();

    let mut all_files: Vec<_> = fs::read_dir(noctua_models_path)?.filter_map(|x| x.ok()).map(|f| f.path()).filter(|f| f.extension().unwrap() == "ttl").collect();
    all_files.sort();
    cam_pipeline_rust::insert_terms_into_graph(&mut output_graph, &all_files, true)?;

    Ok(output_graph)
}
