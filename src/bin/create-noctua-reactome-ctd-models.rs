#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::Graph;
use sophia::triple::stream::TripleSource;
use std::error;
use std::path;
use std::time::Instant;
use structopt::StructOpt;
use walkdir::DirEntry;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-noctua-reactome-ctd-models", about = "create noctua reactome CTD models")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input directory", required = true, parse(from_os_str))]
    input: path::PathBuf,

    #[structopt(short = "t", long = "ctd_tmp_dir", long_help = "ctd-to-owl tmp directory", required = true, parse(from_os_str))]
    ctd_tmp_dir: path::PathBuf,

    #[structopt(short = "o", long = "output", long_help = "output file", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let ttl_files: Vec<DirEntry> = walkdir::WalkDir::new(&options.ctd_tmp_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_name().to_string_lossy().ends_with(".ttl"))
        .collect();

    let mut graph = cam_pipeline_rust::deserialize_graph(&options.input)?;

    for ont in ttl_files.iter() {
        let tmp_graph = cam_pipeline_rust::deserialize_graph(&ont.path().to_path_buf())?;
        tmp_graph.triples().add_to_graph(&mut graph)?;
    }

    let output_path: path::PathBuf = options.output;
    cam_pipeline_rust::serialize_graph(&output_path, &graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
