#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term::TTerm;
use std::error;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create_merged_ontologies", about = "create merged ontologies")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true)]
    input: Vec<path::PathBuf>,

    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let mut graph = FastGraph::new();

    for ont in options.input.iter() {
        let tmp_graph = cam_pipeline_rust::deserialize_graph(&ont)?;
        graph.insert_all(tmp_graph.triples())?;
    }

    // remove --axioms 'disjoint' --trim true --preserve-structure false
    // remove --term 'owl:Nothing' --trim true --preserve-structure false
    // remove --term 'http://purl.obolibrary.org/obo/caro#part_of' --term 'http://purl.obolibrary.org/obo/caro#develops_from' --trim true --preserve-structure false

    let nothing = ns::owl::Nothing;
    let sub_tm = sophia_api::term::matcher::AnyOrExactly::Exactly(nothing);
    let obj_tm = sophia_api::term::matcher::AnyOrExactly::Exactly(nothing);

    let num_of_sub_nothing_removed = graph.remove_matching(&sub_tm, &sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY)?;
    debug!("number of triples removed where subject is {}: {}", nothing.value().to_string(), num_of_sub_nothing_removed);

    let num_of_obj_nothing_removed = graph.remove_matching(&sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY, &obj_tm)?;
    debug!("number of triples removed where object is {}: {}", nothing.value().to_string(), num_of_obj_nothing_removed);

    let caro_ns = ns::Namespace::new("http://purl.obolibrary.org/obo/caro#")?;

    let caro_part_of = caro_ns.get("part_of")?;
    let caro_part_of_tm = sophia_api::term::matcher::AnyOrExactly::Exactly(caro_part_of);

    let num_of_sub_caro_part_of_removed = graph.remove_matching(&caro_part_of_tm, &sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY)?;
    debug!("number of triples removed where subject is {}: {}", caro_part_of.value().to_string(), num_of_sub_caro_part_of_removed);

    let num_of_obj_caro_part_of_removed = graph.remove_matching(&sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY, &caro_part_of_tm)?;
    debug!("number of triples removed where object is {}: {}", caro_part_of.value().to_string(), num_of_obj_caro_part_of_removed);

    let caro_develops_from = caro_ns.get("develops_from")?;
    let caro_develops_from_tm = sophia_api::term::matcher::AnyOrExactly::Exactly(caro_develops_from);

    let num_of_sub_caro_develops_from_removed = graph.remove_matching(&caro_develops_from_tm, &sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY)?;
    debug!("number of triples removed where subject is {}: {}", caro_develops_from.value().to_string(), num_of_sub_caro_develops_from_removed);

    let num_of_obj_caro_develops_from_removed = graph.remove_matching(&sophia_api::term::matcher::ANY, &sophia_api::term::matcher::ANY, &caro_develops_from_tm)?;
    debug!("number of triples removed where object is {}: {}", caro_develops_from.value().to_string(), num_of_obj_caro_develops_from_removed);

    let disjoint_with = ns::owl::disjointWith;
    let disjoint_with_tm = sophia_api::term::matcher::AnyOrExactly::Exactly(disjoint_with);

    let num_of_pred_disjoint_with_removed = graph.remove_matching(&sophia_api::term::matcher::ANY, &disjoint_with_tm, &sophia_api::term::matcher::ANY)?;
    debug!("number of triples removed where predicate is {}: {}", disjoint_with.value().to_string(), num_of_pred_disjoint_with_removed);

    cam_pipeline_rust::serialize_graph(&options.output, &graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
