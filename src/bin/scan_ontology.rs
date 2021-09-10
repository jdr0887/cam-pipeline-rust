#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::Graph;
use sophia::ns;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use std::error;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "scan_ontology", about = "scan ontology")]
struct Options {
    #[structopt(short = "i", long = "input", long_help = "input", required = true, parse(from_os_str))]
    input: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let graph = cam_pipeline_rust::deserialize_graph(&options.input)?;

    // let metatype_ns = ns::Namespace::new("https://w3id.org/biolink/biolinkml/meta/types/")?;
    let metatype_ns = ns::Namespace::new("https://w3id.org/linkml/")?;
    let metatype_class_definition = metatype_ns.get("ClassDefinition")?;

    graph.triples_with_po(&ns::rdf::type_, &metatype_class_definition).for_each_triple(|t| {
        // graph.triples_with_o(&metatype_class_definition).for_each_triple(|t| {
        let obj = t.o().value().to_string();
        let pred = t.p().value().to_string();
        let sub = t.s().value().to_string();
        debug!("sub: {:?}, pred: {:?} obj: {:?}", sub, pred, obj);
    })?;

    // let ncbigene_ns = ns::Namespace::new("http://purl.obolibrary.org/obo/HP_")?;
    // let gene = ncbigene_ns.get("0002013")?;
    //
    // graph.triples().filter_triples(|t| t.s().value().to_string() == gene.to_string() || t.o().value().to_string() == gene.to_string()).for_each_triple(|t| {
    //     let obj = t.o().value().to_string();
    //     let pred = t.p().value().to_string();
    //     let sub = t.s().value().to_string();
    //     debug!("sub: {:?}, pred: {:?} obj: {:?}", sub, pred, obj);
    // })?;

    // let biolink_ns = ns::Namespace::new("https://w3id.org/biolink/vocab/")?;
    // let biolink_pathway = biolink_ns.get("Pathway")?;
    // let biolink_gene_or_gene_product = biolink_ns.get("GeneOrGeneProduct")?;
    //
    // cam_reasoned_graph.triples_with_po(&ns::rdfs::subClassOf, &biolink_pathway).for_each_triple(|t| {
    //     let obj = t.o().value().to_string();
    //     let sub = t.s().value().to_string();
    //     debug!("sub: {:?}, obj: {:?}", sub, obj);
    // })?;

    // let ncbigene_ns = ns::Namespace::new("http://identifiers.org/ncbigene/")?;
    // let gene = ncbigene_ns.get("6658")?;

    // let pubchem_compound_ns = ns::Namespace::new("http://identifiers.org/pubchem.compound/")?;
    // let pubchem_compound = pubchem_compound_ns.get("5291")?;
    // let pubchem_compound = pubchem_compound_ns.get("2662")?;

    // cam_reasoned_graph
    //     .triples()
    //     .filter_triples(|t| t.s().value().to_string() == pubchem_compound.to_string() || t.o().value().to_string() == pubchem_compound.to_string())
    //     .for_each_triple(|t| {
    //         let obj = t.o().value().to_string();
    //         let pred = t.p().value().to_string();
    //         let sub = t.s().value().to_string();
    //         debug!("sub: {:?}, pred: {:?} obj: {:?}", sub, pred, obj);
    //     })?;

    // let output_path: path::PathBuf = path::PathBuf::new().join("/home/jdr0887/results.txt");
    // let mut writer = io::BufWriter::new(fs::File::create(output_path.as_path())?);
    // for (key, value) in results.iter() {
    //     for v in value.iter() {
    //         writer.write_all(format!("{},id_prefixes,{}\n", key, v).as_bytes()).expect("Unable to write data");
    //     }
    // }

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
