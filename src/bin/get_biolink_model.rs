#[macro_use]
extern crate log;

use humantime::format_duration;
use std::error;
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
