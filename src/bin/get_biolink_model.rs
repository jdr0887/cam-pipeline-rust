#[macro_use]
extern crate log;

use humantime::format_duration;
use std::error;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "get_biolink_model", about = "get biolink model")]
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

    let biolink_model_path: path::PathBuf = work_dir.clone().join("biolink-model.ttl");
    cam_pipeline_rust::get_biolink_model(&biolink_model_path)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
