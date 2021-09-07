#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::error;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "construct_is_defined_by", about = "construct is_defined_by")]
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
    let merged_ontologies_path: path::PathBuf = work_dir.clone().join("merged-ontologies.nt");
    let merged_ontologies_graph = cam_pipeline_rust::deserialize_graph(&merged_ontologies_path)?;

    let output_graph = is_defined_by(&merged_ontologies_graph)?;

    let output_path: path::PathBuf = work_dir.clone().join("is-defined-by.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

pub fn is_defined_by(merged_ontologies_graph: &FastGraph) -> Result<FastGraph, Box<dyn error::Error>> {
    let mut output_graph = FastGraph::new();

    let prefix = "http://purl.obolibrary.org/obo/";
    let re = regex::Regex::new(format!("{}(.+_.+)", prefix).as_str())?;

    merged_ontologies_graph.triples().filter_triples(|t| t.s().value().to_string().starts_with(prefix)).for_each_triple(|t| {
        let sub = t.s().value().to_string();

        if re.is_match(&sub) {
            let capture = re.captures_iter(&sub).next().unwrap();
            let suffix = capture.get(1).unwrap().as_str();
            let idx = suffix.rfind("_").unwrap();
            let trimmed_suffix = suffix.split_at(idx).0;
            let mut value = prefix.to_string();
            value.push_str(trimmed_suffix);
            value.push_str(".owl");
            let iri = value.to_lowercase();
            let ont_term = term::SimpleIri::new(iri.as_str(), None).unwrap();
            // debug!("sub: {}, ont_term: {}", sub, ont_term.value().to_string());
            output_graph.insert(t.s(), &ns::rdfs::isDefinedBy, &ont_term).unwrap();
        }
    })?;

    Ok(output_graph)
}
