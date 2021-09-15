#[macro_use]
extern crate log;

use std::error;
use std::fs;
use std::io;
use std::io::{BufRead, Error, Write};
use std::ops::Deref;
use std::path;
use std::time;

use crepe::crepe;
use env_logger;
use humantime::format_duration;
use itertools::Itertools;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "reasoner", about = "reasoner")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,

    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = time::Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let input_file = fs::File::open(&options.input)?;
    let br = io::BufReader::new(&input_file);

    let output_file = fs::File::create(&options.output)?;
    let mut bw = io::BufWriter::new(&output_file);

    for line in br.lines() {
        let line = line.unwrap();
        let converted_to_fact = line.replace(" ", "\t");
        let converted_to_fact = converted_to_fact.trim_end_matches(".");
        bw.write_all(format!("{}\n", converted_to_fact).as_bytes());
    }
    Ok(())
}
