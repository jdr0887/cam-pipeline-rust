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

    //prp-spo2
    //optimize by only creating chains after two hops?
    // chain(?s, ?p, ?o, ?y) :- rdf(?p, "<http://www.w3.org/2002/07/owl#propertyChainAxiom>", ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?s, ?p1, ?o), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
    // chain(?s, ?p, ?o2, ?y) :- chain(?s, ?p, ?o, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?o, ?p1, ?o2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
    // rdf(?s, ?p, ?o2) :- chain(?s, ?p, ?o, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?o, ?p1, ?o2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>").

    //prp-eqp1
    // rdf(?x, ?p2, ?y) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", ?p2), rdf(?x, ?p1, ?y).
    Reachable(x, p2, y) <- WhelkRDF(p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", p2), WhelkRDF(x, p1, y);

    //prp-eqp2
    // rdf(?x, ?p1, ?y) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", ?p2), rdf(?x, ?p2, ?y).
    Reachable(x, p1, y) <- WhelkRDF(p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", p2), WhelkRDF(x, p2, y);

    //prp-pdw
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
    // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p1, "<http://www.w3.org/2002/07/owl#propertyDisjointWith>", ?p2), rdf(?x, ?p1, ?y), rdf(?x, ?p2, ?y).

    //prp-adp
    // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
    // rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointProperties>"), rdf(?x, "<http://www.w3.org/2002/07/owl#members>", ?y), listContains(?y, ?pi), listContains(?y, ?pj), ?pi != ?pj, rdf(?u, ?pi, ?v), rdf(?u, ?pj, ?v).

    //prp-inv1
    //rdf(?y, ?p2, ?x) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#inverseOf>", ?p2), rdf(?x, ?p1, ?y).
    Reachable(y, p2, x) <- WhelkRDF(p1, "<http://www.w3.org/2002/07/owl#inverseOf>", p2), WhelkRDF(x, p1, y);

    //prp-inv2
    //rdf(?y, ?p1, ?x) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#inverseOf>", ?p2), rdf(?x, ?p2, ?y).
    Reachable(y, p1, x) <- WhelkRDF(p1, "<http://www.w3.org/2002/07/owl#inverseOf>", p2), WhelkRDF(x, p2, y);

    //prp-key
    //TODO

    //prp-npa1
    // rdf(?i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
    // rdf(?i2, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", ?i1), rdf(?x, "<http://www.w3.org/2002/07/owl#assertionProperty>", ?p), rdf(?x, "<http://www.w3.org/2002/07/owl#targetIndividual>", ?i2), rdf(?i1, ?p, ?i2).

    //prp-npa2
    // rdf(?i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", ?i1), rdf(?x, "<http://www.w3.org/2002/07/owl#assertionProperty>", ?p), rdf(?x, "<http://www.w3.org/2002/07/owl#targetValue>", ?lt), rdf(?i1, ?p, ?lt).

    //cls-int1
    //optimize by only creating ichains after two hops?
    // ichain(?i, ?c, ?y) :- rdf(?c, "<http://www.w3.org/2002/07/owl#intersectionOf>", ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
    // ichain(?i, ?c, ?y) :- ichain(?i, ?c, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
    // rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- ichain(?i, ?c, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>").

    //cls-int2
    // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1) :- rdf(?c, "<http://www.w3.org/2002/07/owl#intersectionOf>", ?x), listContains(?x, ?c1), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c).

    //cls-uni
    // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?c, "<http://www.w3.org/2002/07/owl#unionOf>", ?x), listContains(?x, ?c1), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).

    //cls-com
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?c1, "<http://www.w3.org/2002/07/owl#complementOf>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).

    //cls-svf1
    // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, ?v), rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?y).

    //cls-svf2
    // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", "<http://www.w3.org/2002/07/owl#Thing>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, _).

    //cls-avf
    // rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?y) :- rdf(?x, "<http://www.w3.org/2002/07/owl#allValuesFrom>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, ?v).

    //cls-hv1
    // rdf(?u, ?p, ?y) :- rdf(?x, "<http://www.w3.org/2002/07/owl#hasValue>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x).

    //cls-hv2
    // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#hasValue>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, ?y).
    Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x) <- WhelkRDF(x, "<http://www.w3.org/2002/07/owl#hasValue>", y), WhelkRDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), WhelkRDF(u, p, y);

    //cls-maxc1
    // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"0\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, _).

    //cls-maxc2
    // rdf(?y1, "<http://www.w3.org/2002/07/owl#sameAs>", ?y2) :- rdf(?x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"1\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, ?y1), rdf(?u, ?p, ?y2).

    //cls-maxqc1
    //TODO

    //cls-maxqc2
    //TODO

    //cls-maxqc3
    //TODO

    //cls-maxqc4
    //TODO

    //cls-oo
    // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?c, "<http://www.w3.org/2002/07/owl#oneOf>", ?x), listContains(?x, ?y).

    //cax-sco
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2) :- rdf(?c1, "<http://www.w3.org/2000/01/rdf-schema#subClassOf?", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).
    Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2) <- WhelkRDF(c1, "<http://www.w3.org/2000/01/rdf-schema#subClassOf?", c2), WhelkRDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1);

    //cax-eqc1
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2) :- rdf(?c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).

    //cax-eqc2
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1) :- rdf(?c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).

    //cax-dw
    // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?c1, "<http://www.w3.org/2002/07/owl#disjointWith>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).

    //cax-adc
    // rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointClasses>"), rdf(?x, "<http://www.w3.org/2002/07/owl#members>", ?y), listContains(?y, ?ci), listContains(?y, ?cj), ?ci != ?cj, rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?ci), rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?cj).

}

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
