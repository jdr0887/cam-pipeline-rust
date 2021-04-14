#[macro_use]
extern crate log;

use humantime::format_duration;
use sophia::graph::inmem::FastGraph;
use sophia::graph::{Graph, MutableGraph};
use sophia::ns;
use sophia::term::TTerm;
use sophia::triple::stream::TripleSource;
use sophia::triple::Triple;
use sophia_api::term::matcher::TermMatcher;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    env_logger::init();

    let base_path: path::PathBuf = path::PathBuf::new().join("src/data");

    // let noctua_reactome_ontology_path: path::PathBuf = base_path.clone().join("noctua-reactome-ontology.nt");
    // let biolink_model_path: path::PathBuf = base_path.clone().join("biolink-model.ttl");
    // let biolink_local_path: path::PathBuf = base_path.clone().join("biolink-local.ttl");

    let ontologies_path: path::PathBuf = base_path.clone().join("ontologies.nt");
    let ubergraph_axioms_path: path::PathBuf = base_path.clone().join("ubergraph-axioms.ttl");
    let uniprot_to_ncbi_rules_path: path::PathBuf = base_path.clone().join("uniprot-to-ncbi-rules.ttl");
    let mesh_chebi_links_path: path::PathBuf = base_path.clone().join("mesh-chebi-links.nt");
    let protein_subclasses_path: path::PathBuf = base_path.clone().join("protein-subclasses.nt");
    let reacto_uniprot_rules_path: path::PathBuf = base_path.clone().join("reacto-uniprot-rules.nt");
    let ncbi_gene_classes_path: path::PathBuf = base_path.clone().join("ncbi-gene-classes.nt");
    let ont_biolink_subclasses_path: path::PathBuf = base_path.clone().join("ont-biolink-subclasses.nt");
    let slot_mappings_path: path::PathBuf = base_path.clone().join("slot-mappings.nt");
    let biolink_class_hierarchy_path: path::PathBuf = base_path.clone().join("biolink-class-hierarchy.nt");

    let files_to_merge = vec![
        ontologies_path,
        ubergraph_axioms_path,
        uniprot_to_ncbi_rules_path,
        mesh_chebi_links_path,
        protein_subclasses_path,
        reacto_uniprot_rules_path,
        ncbi_gene_classes_path,
        ont_biolink_subclasses_path,
        slot_mappings_path,
        biolink_class_hierarchy_path,
    ];

    let mut graph = FastGraph::new();

    for ont in files_to_merge.iter() {
        let tmp_graph = cam_pipeline_rust::deserialize_graph(&ont)?;
        tmp_graph.triples().add_to_graph(&mut graph)?;
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

    let output_path: path::PathBuf = base_path.clone().join("merged-ontologies.nt");
    cam_pipeline_rust::serialize_graph(&output_path, &graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
