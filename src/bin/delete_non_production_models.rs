#[macro_use]
extern crate log;

use humantime::format_duration;
use oxigraph::io::GraphFormat;
use oxigraph::model::GraphName;
use std::error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "delete_non_production_models", about = "delete non production models")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let ont_graph = cam_pipeline_rust::deserialize_graph(&options.input)?;

    let store = cam_pipeline_rust::load_graphs_into_memory_store(vec![ont_graph])?;
    store.update(include_str!("../sparql/delete-non-production-models.ru"))?;

    let output_file = fs::File::create(&options.input)?;
    let mut writer = io::BufWriter::new(output_file);
    store.dump_graph(&mut writer, GraphFormat::NTriples, &GraphName::DefaultGraph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
