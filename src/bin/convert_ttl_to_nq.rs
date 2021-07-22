#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::parser;
use sophia::serializer::nq::NqSerializer;
use sophia::serializer::{QuadSerializer, TripleSerializer};
use sophia::triple::stream::TripleSource;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let start = Instant::now();

    // let reader = io::BufReader::new(fs::File::open(&options.input)?);
    // let triple_source = parser::turtle::parse_bufread(reader);
    // let graph: FastGraph = triple_source.collect_triples().unwrap();
    //
    // let writer = io::BufWriter::new(fs::File::create(&options.output)?);
    // let mut serializer = sophia::serializer::nq::NqSerializer::new(writer);
    // serializer.serialize_dataset(&graph.as_dataset())?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
