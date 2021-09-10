#[macro_use]
extern crate log;

use std::error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

use humantime::format_duration;
use oxigraph::io::GraphFormat;

#[derive(StructOpt, Debug)]
#[structopt(name = "construct_biolink_class_hierarchy", about = "construct biolink class hierarchy")]
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

    let biolink_model_graph = cam_pipeline_rust::deserialize_graph(&options.input)?;
    let output_file = fs::File::create(&options.output)?;
    let mut writer = io::BufWriter::new(output_file);
    let store = cam_pipeline_rust::load_graphs_into_memory_store(vec![biolink_model_graph])?;
    let results = store.query(include_str!("../sparql/construct-biolink-class-hierachy.rq"))?;
    results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
