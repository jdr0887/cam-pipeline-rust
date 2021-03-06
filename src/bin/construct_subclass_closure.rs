#[macro_use]
extern crate log;

use humantime::format_duration;
use oxigraph::io::GraphFormat;
use std::error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "construct_subclass_closure", about = "construct subclass closure")]
struct Options {
    #[structopt(short = "w", long = "work_dir", long_help = "work directory", required = true, parse(from_os_str))]
    work_dir: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let work_dir: path::PathBuf = options.work_dir;

    let merged_ontologies_path = work_dir.clone().join("merged-ontologies.nt");
    let store = get_store(&merged_ontologies_path)?;
    let results = store.query(include_str!("../sparql/subclass-closure.rq"))?;

    let output_path: path::PathBuf = work_dir.clone().join("subclass-closure.nt");
    let output_file = fs::File::create(&output_path)?;
    let mut writer = io::BufWriter::new(output_file);
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn get_store(merged_ontologies_path: &path::PathBuf) -> Result<oxigraph::MemoryStore, Box<dyn error::Error>> {
    let merged_ontologies_graph = cam_pipeline_rust::deserialize_graph(&merged_ontologies_path)?;
    let store = cam_pipeline_rust::load_graphs_into_memory_store(vec![merged_ontologies_graph])?;
    Ok(store)
}
