#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::term::TTerm;
use std::error;
use std::fs;
use std::path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let base_path: path::PathBuf = path::PathBuf::new().join("src/data");

    let noctua_models_path: path::PathBuf = path::PathBuf::new().join("../noctua-models-prod/models");
    let noctua_models_dev_path: path::PathBuf = path::PathBuf::new().join("../noctua-models-dev/models");

    let output_graph = noctua_reactome_ontology(&noctua_models_path, &noctua_models_dev_path)?;

    let output_path: path::PathBuf = base_path.clone().join("noctua-reactome-ontology.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn noctua_reactome_ontology(noctua_models_path: &path::PathBuf, noctua_models_dev_path: &path::PathBuf) -> Result<FastGraph, Box<dyn error::Error>> {
    let mut output_graph = FastGraph::new();

    let mut all_files: Vec<_> = fs::read_dir(noctua_models_path)?.filter_map(|x| x.ok()).map(|f| f.path()).filter(|f| f.extension().unwrap() == "ttl").collect();
    all_files.sort();
    cam_pipeline_rust::insert_terms_into_graph(&mut output_graph, &all_files, true)?;

    let mut all_files: Vec<_> = fs::read_dir(noctua_models_dev_path)?
        .filter_map(|x| x.ok())
        .map(|f| f.path())
        .filter(|f| f.extension().unwrap() == "ttl" && f.file_name().unwrap().to_str().unwrap().starts_with("R-HSA-"))
        .collect();
    all_files.sort();
    cam_pipeline_rust::insert_terms_into_graph(&mut output_graph, &all_files, false)?;

    Ok(output_graph)
}
