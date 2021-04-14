#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::parser;
use sophia::serializer::TripleSerializer;
use sophia::triple::stream::TripleSource;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "convert_owl_to_nt", about = "Convert owl to nt")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,

    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let start = Instant::now();
    let options = Options::from_args();
    debug!("{:?}", options);

    let reader = io::BufReader::new(fs::File::open(&options.input)?);
    let triple_source = parser::xml::parse_bufread(reader);
    let graph: FastGraph = triple_source.collect_triples().unwrap();

    let writer = io::BufWriter::new(fs::File::create(&options.output)?);
    let mut serializer = sophia::serializer::nt::NtSerializer::new(writer);
    serializer.serialize_graph(&graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
