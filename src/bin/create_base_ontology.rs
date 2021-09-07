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
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create_base_ontology", about = "create base ontology")]
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

    let ontologies_path: path::PathBuf = work_dir.clone().join("ontologies.ttl");
    let output_graph = base_ontology(&ontologies_path)?;
    let output_path: path::PathBuf = work_dir.clone().join("ontologies.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

fn base_ontology(ontologies_path: &path::PathBuf) -> Result<FastGraph, Box<dyn error::Error>> {
    let mut output_graph = FastGraph::new();

    let graph = cam_pipeline_rust::deserialize_graph(&ontologies_path)?;

    let owl_ns = ns::Namespace::new("http://www.w3.org/2002/07/owl#")?;
    let owl_ontology = owl_ns.get("Ontology")?;
    let owl_imports = owl_ns.get("imports")?;

    let potential_base_ontology_triples: Vec<String> =
        graph.triples_with_po(&ns::rdf::type_, &owl_ontology).map_triples(|t| t.s().value().to_string()).into_iter().collect::<Result<Vec<String>, _>>()?;

    let base_ontology = term::SimpleIri::new(potential_base_ontology_triples.iter().next().unwrap(), None).unwrap();
    debug!("base_ontology: {:?}", base_ontology);

    let home_dir = dirs::home_dir().unwrap();
    let owl_import_dir = home_dir.clone().join(".owl");
    if !owl_import_dir.exists() {
        fs::create_dir_all(&owl_import_dir)?;
    }

    graph.triples_with_p(&owl_imports).for_each_triple(|t| {
        let import = t.o().value().to_string();
        let owl_url = import.replace("http://", "").replace("<", "").replace(">", "");
        let owl_on_fs = owl_import_dir.clone().join(owl_url);
        if !owl_on_fs.exists() {
            info!("importing: {}", &import);
            fs::create_dir_all(&owl_on_fs.parent().expect("could not get parent directory")).unwrap();
            let response = ureq::get(&import).call().unwrap();
            let data = response.into_string().unwrap();
            let output = fs::File::create(&owl_on_fs).unwrap();
            let mut tmp_writer = io::BufWriter::new(output);
            tmp_writer.write_all(data.as_bytes()).expect("Unable to write data");
        }
        let tmp_graph = cam_pipeline_rust::deserialize_graph(&owl_on_fs).unwrap();
        tmp_graph
            .triples()
            .for_each_triple(|inner_triple| {
                if inner_triple.s().kind() == term::TermKind::BlankNode && inner_triple.o().kind() == term::TermKind::BlankNode {
                    let sub = inner_triple.s().value().to_string().replace("riog", owl_on_fs.file_stem().unwrap().to_str().unwrap());
                    let sub_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(sub.as_str()).unwrap();

                    let obj = inner_triple.o().value().to_string().replace("riog", owl_on_fs.file_stem().unwrap().to_str().unwrap());
                    let obj_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(obj.as_str()).unwrap();

                    output_graph.insert(&sub_term, inner_triple.p(), &obj_term).unwrap();
                    return;
                }

                if inner_triple.s().kind() == term::TermKind::BlankNode && inner_triple.o().kind() != term::TermKind::BlankNode {
                    let sub = inner_triple.s().value().to_string().replace("riog", owl_on_fs.file_stem().unwrap().to_str().unwrap());
                    let sub_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(sub.as_str()).unwrap();

                    output_graph.insert(&sub_term, inner_triple.p(), inner_triple.o()).unwrap();
                    return;
                }

                if inner_triple.s().kind() != term::TermKind::BlankNode && inner_triple.o().kind() == term::TermKind::BlankNode {
                    let obj = inner_triple.o().value().to_string().replace("riog", owl_on_fs.file_stem().unwrap().to_str().unwrap());
                    let obj_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(obj.as_str()).unwrap();

                    output_graph.insert(inner_triple.s(), inner_triple.p(), &obj_term).unwrap();
                    return;
                }

                output_graph.insert(inner_triple.s(), inner_triple.p(), inner_triple.o()).unwrap();
            })
            .unwrap();
        debug!("tmp_graph.triples().count(): {}, output_graph.triples().count(): {}", tmp_graph.triples().count(), output_graph.triples().count());
    })?;

    graph.triples().filter_triples(|t| t.p() != &owl_imports).for_each_triple(|triple| {
        if triple.s().kind() == term::TermKind::BlankNode && triple.o().kind() == term::TermKind::BlankNode {
            let sub = triple.s().value().to_string().replace("riog", "cam");
            let sub_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(sub.as_str()).unwrap();

            let obj = triple.o().value().to_string().replace("riog", "cam");
            let obj_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(obj.as_str()).unwrap();

            output_graph.insert(&sub_term, triple.p(), &obj_term).unwrap();
            return;
        }

        if triple.s().kind() == term::TermKind::BlankNode && triple.o().kind() != term::TermKind::BlankNode {
            let sub = triple.s().value().to_string().replace("riog", "cam");
            let sub_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(sub.as_str()).unwrap();

            output_graph.insert(&sub_term, triple.p(), triple.o()).unwrap();
            return;
        }

        if triple.s().kind() != term::TermKind::BlankNode && triple.o().kind() == term::TermKind::BlankNode {
            let obj = triple.o().value().to_string().replace("riog", "cam");
            let obj_term: term::blank_node::BlankNode<&str> = term::blank_node::BlankNode::new(obj.as_str()).unwrap();

            output_graph.insert(triple.s(), triple.p(), &obj_term).unwrap();
            return;
        }

        output_graph.insert(triple.s(), triple.p(), triple.o()).unwrap();
    })?;

    Ok(output_graph)
}
