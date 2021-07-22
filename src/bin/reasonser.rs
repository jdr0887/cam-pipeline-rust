#[macro_use]
extern crate log;

use crepe::crepe;
use env_logger;
use humantime::format_duration;
use itertools::Itertools;
use std::error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path;
use std::time;
use structopt::StructOpt;

crepe! {
    @input
    struct WhelkRDF<'a>(&'a str, &'a str, &'a str);

    @output
    struct Reachable<'a>(&'a str, &'a str, &'a str);

    //prp-dom
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?p, "<http://www.w3.org/2000/01/rdf-schema#domain>", ?c), rdf(?x, ?p, _).
    Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- WhelkRDF(p, "<http://www.w3.org/2000/01/rdf-schema#domain>", c), WhelkRDF(x, p, _);

    //prp-rng
    // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?p, "<http://www.w3.org/2000/01/rdf-schema#range>", ?c), rdf(_, ?p, ?y).
    Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- WhelkRDF(p, "<http://www.w3.org/2000/01/rdf-schema#range>", c), WhelkRDF(_, p, y);

    //prp-fp
    // rdf(?y1, "<http://www.w3.org/2002/07/owl#sameAs>", ?y2) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), rdf(?x, ?p, ?y1), rdf(?x, ?p, ?y2).
    Reachable(y1, "<http://www.w3.org/2002/07/owl#sameAs>", y2) <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), WhelkRDF(x, p, y1), WhelkRDF(x, p, y2);

    //prp-ifp
    // rdf(?x1, "<http://www.w3.org/2002/07/owl#sameAs>", ?x2) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#InverseFunctionalProperty>"), rdf(?x1, ?p, ?y), rdf(?x2, ?p, ?y).
    Reachable(x1, "<http://www.w3.org/2002/07/owl#sameAs>", x2) <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), WhelkRDF(x1, p, y), WhelkRDF(x2, p, y);

    //prp-irp
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#IrreflexiveProperty>"), rdf(?x, ?p, ?x).
    // Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#IrreflexiveProperty>"), WhelkRDF(x, p, x);

    //prp-symp
    // rdf(?y, ?p, ?x) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#SymmetricProperty>"), rdf(?x, ?p, ?y).
    Reachable(y, p, x) <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#SymmetricProperty>"), WhelkRDF(x, p, y);

    //prp-asyp
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), rdf(?x, ?p, ?y), rdf(?y, ?p, ?x).
    Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), WhelkRDF(x, p, y), WhelkRDF(y, p, x);
    Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), WhelkRDF(x, p, y), WhelkRDF(y, p, x);

    //prp-trp
    // rdf(?x, ?p, ?z) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#TransitiveProperty>"), rdf(?x, ?p, ?y), rdf(?y, ?p, ?z).
    Reachable(x, p, z) <- WhelkRDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#TransitiveProperty>"), WhelkRDF(x, p, y), WhelkRDF(y, p, z);

    //prp-spo1
    // rdf(?x, ?p2, ?y) :- rdf(?p1, "<http://www.w3.org/2000/01/rdf-schema#subPropertyOf>", ?p2), rdf(?x, ?p1, ?y).
    Reachable(x, p2, y) <- WhelkRDF(p1, "<http://www.w3.org/2000/01/rdf-schema#subPropertyOf>", p2), WhelkRDF(x, p1, y);

}

#[derive(StructOpt, Debug)]
#[structopt(name = "convert_owl_to_nt", about = "Convert owl to nt")]
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
    let mut runtime = Crepe::new();

    let data = convert_to_rdf(&options.input)?;
    runtime.extend(data.iter().map(|(a, b, c)| WhelkRDF(a.as_str(), b.as_str(), c.as_str())));

    let (reachable,) = runtime.run();
    for Reachable(x, y, z) in reachable {
        info!("{} {} {}", x, y, z);
    }
    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn convert_to_rdf<'a>(input_path: &path::Path) -> Result<Vec<(String, String, String)>, Box<dyn error::Error>> {
    let input_file = fs::File::open(input_path)?;
    let br = io::BufReader::new(input_file);

    let mut data = Vec::new();

    for line in br.lines() {
        let unwrapped = line.unwrap();
        if !unwrapped.contains("\"") {
            let split = unwrapped.split_whitespace().collect_vec();
            data.push((split[0].to_string(), split[1].to_string(), split[2].to_string()));
        }
    }

    Ok(data)
}
