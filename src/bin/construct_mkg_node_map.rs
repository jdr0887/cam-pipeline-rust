#[macro_use]
extern crate log;

use humantime::format_duration;
use itertools::Itertools;
use rayon::prelude::*;
use sophia::graph::Graph;
use sophia::ns;
use sophia::term::{TTerm, TermKind};
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fs;
use std::io;
use std::io::prelude::Write;
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
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();
    let options = Options::from_args();
    debug!("{:?}", options);

    let prefixes_map: HashMap<String, String> = include_str!("../../src/data/prefixes.csv")
        .lines()
        .map(|line| line.split(",").map(str::to_owned).collect_vec())
        .map(|vec| {
            assert_eq!(vec.len(), 2);
            (vec[1].to_string(), vec[0].to_string())
        })
        .collect();

    let cam_reasoned_graph = cam_pipeline_rust::deserialize_graph(&options.input)?;

    let mut results: HashMap<String, HashSet<String>> = HashMap::new();

    cam_reasoned_graph
        .triples_with_p(&ns::rdfs::subClassOf)
        .filter_triples(|t| t.s().kind() == TermKind::Iri)
        .filter_triples(|t| t.o().kind() == TermKind::Iri)
        .filter_triples(|t| !t.s().value().to_string().starts_with("https://w3id.org/biolink/vocab/"))
        .filter_triples(|t| t.o().value().to_string().starts_with("https://w3id.org/biolink/vocab/"))
        .for_each_triple(|t| {
            let obj = t.o().value().to_string();
            let sub = t.s().value().to_string();
            let found = prefixes_map.par_iter().filter(|(k, _v)| sub.starts_with(*k)).map(|(k, _v)| k).max_by(|a, b| a.len().cmp(&b.len()));
            match found {
                Some(k) => {
                    let v = prefixes_map.get(k).unwrap();
                    debug!("adding v: {:?} to obj: {:?}", v, obj);
                    results.entry(obj).or_insert(HashSet::new()).insert(v.to_string());
                }
                None => {
                    warn!("prefix not found - sub: {:?}, obj: {:?}", sub, obj);
                }
            }
        })?;

    let mut writer = io::BufWriter::new(fs::File::create(&options.output.as_path())?);
    for (key, value) in results.iter() {
        for v in value.iter() {
            writer.write_all(format!("{},id_prefixes,{}\n", key, v).as_bytes()).expect("Unable to write data");
        }
    }

    // let store = get_store(&cam_reasoned_ttl_path)?;
    // info!("running query");
    // if let QueryResults::Solutions(mut solutions) = store.query(include_str!("../sparql/find-prefix-mappings.rq"))? {
    //     solutions.for_each(|a| {
    //         let qs = a.unwrap();
    //         let bl_category = qs.get("blcategory");
    //         let prefix = qs.get("prefix");
    //         let expansion = qs.get("expansion");
    //         info!("bl_category: {:?}, prefix: {:?}, expansion: {:?}", bl_category, prefix, expansion);
    //     });
    // }

    // let output_path: path::PathBuf = path::PathBuf::new().join("src/data/prefix-mappings.nt");
    // let output_file = fs::File::create(&output_path)?;
    // let mut writer = io::BufWriter::new(output_file);
    // results.write_graph(&mut writer, GraphFormat::NTriples)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

#[allow(dead_code)]
fn get_store(cam_reasoned_path: &path::PathBuf) -> Result<oxigraph::RocksDbStore, Box<dyn error::Error>> {
    let cam_reasoned_db_path = path::PathBuf::new().join("/home/jdr0887/cam-db-reasoned");
    if cam_reasoned_db_path.exists() {
        let store = oxigraph::RocksDbStore::open(&cam_reasoned_db_path)?;
        Ok(store)
    } else {
        fs::create_dir_all(cam_reasoned_db_path.as_path())?;
        let cam_reasoned_graph = cam_pipeline_rust::deserialize_graph(&cam_reasoned_path)?;
        let store = cam_pipeline_rust::load_graphs_into_rocksdb_store(&cam_reasoned_db_path, vec![cam_reasoned_graph])?;
        Ok(store)
    }
}
