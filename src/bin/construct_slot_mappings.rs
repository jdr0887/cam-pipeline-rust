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
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,

    #[structopt(short = "b", long = "biolink_model", long_help = "biolink model", required = true, parse(from_os_str))]
    biolink_model: path::PathBuf,

    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let biolink_model_graph = cam_pipeline_rust::deserialize_graph(&options.biolink_model)?;
    let biolink_local_graph = cam_pipeline_rust::deserialize_graph(&options.input)?;

    let output_file = fs::File::create(&options.output)?;
    let mut writer = io::BufWriter::new(output_file);
    let store = cam_pipeline_rust::load_graphs_into_memory_store(vec![biolink_model_graph, biolink_local_graph])?;
    let results = store.query(include_str!("../../src/sparql/construct-slot-mappings.rq"))?;
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
