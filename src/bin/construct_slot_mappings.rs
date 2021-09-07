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
#[structopt(name = "construct_slot_mappings", about = "construct slot mappings")]
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

    let biolink_local_path: path::PathBuf = work_dir.clone().join("biolink-local.ttl");
    let biolink_local_graph = cam_pipeline_rust::deserialize_graph(&biolink_local_path)?;

    let biolink_model_path: path::PathBuf = work_dir.clone().join("biolink-model.ttl");
    let biolink_model_graph = cam_pipeline_rust::get_biolink_model(&biolink_model_path)?;

    let output_path: path::PathBuf = work_dir.clone().join("slot-mappings.nt");
    let output_file = fs::File::create(&output_path)?;
    let mut writer = io::BufWriter::new(output_file);
    let store = cam_pipeline_rust::load_graphs_into_memory_store(vec![biolink_model_graph, biolink_local_graph])?;
    let results = store.query(include_str!("../../src/sparql/construct-slot-mappings.rq"))?;
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
