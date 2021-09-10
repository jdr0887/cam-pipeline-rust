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
use std::path;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "construct_reacto_uniprot_rules", about = "construct REACTO UNIPROT rules")]
struct Options {
    #[structopt(short = "o", long = "output", long_help = "output", required = true, parse(from_os_str))]
    output: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let home_dir = dirs::home_dir().unwrap();
    let owl_import_dir = home_dir.clone().join(".owl");
    if !owl_import_dir.exists() {
        fs::create_dir_all(&owl_import_dir)?;
    }

    let reacto_ont_path: path::PathBuf = owl_import_dir.clone().join("purl.obolibrary.org/obo/go/snapshot/extensions/reacto.owl");
    let reacto_ont_graph = cam_pipeline_rust::deserialize_graph(&reacto_ont_path)?;

    let output_graph = reacto_uniprot_rules(&reacto_ont_graph)?;

    cam_pipeline_rust::serialize_graph(&options.output, &output_graph)?;

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}

// PREFIX go: <http://www.geneontology.org/formats/oboInOwl#>
// PREFIX blml: <https://w3id.org/biolink/biolinkml/meta/>
// PREFIX bl: <https://w3id.org/biolink/vocab/>
// PREFIX MONDO: <http://purl.obolibrary.org/obo/MONDO_>
// PREFIX SO: <http://purl.obolibrary.org/obo/SO_>
// PREFIX RO: <http://purl.obolibrary.org/obo/RO_>
// PREFIX GO: <http://purl.obolibrary.org/obo/GO_>
// PREFIX obo: <http://purl.obolibrary.org/obo/>
// PREFIX NCBIGENE: <http://identifiers.org/ncbigene:>
// PREFIX sesame: <http://www.openrdf.org/schema/sesame#>
// PREFIX swrl: <http://www.w3.org/2003/11/swrl#>
// CONSTRUCT {
//     <urn:swrl#x> rdf:type swrl:Variable .
//     ?u rdf:type owl:Class .
//     ?u rdfs:subClassOf SO:0000704 .
//     [
//         rdf:type <http://www.w3.org/2003/11/swrl#Imp> ;
//         swrl:body [
//           rdf:type swrl:AtomList ;
//           rdf:first [
//             rdf:type swrl:ClassAtom ;
//             swrl:classPredicate ?r ;
//             swrl:argument1 <urn:swrl#x>
//           ] ;
//           rdf:rest rdf:nil
//         ] ;
//         swrl:head [
//           rdf:type swrl:AtomList ;
//           rdf:first [
//             rdf:type swrl:ClassAtom ;
//             swrl:classPredicate ?u ;
//             swrl:argument1 <urn:swrl#x>
//           ] ;
//           rdf:rest rdf:nil
//         ]
//     ]
//   }
//   WHERE {
//   ?r <http://geneontology.org/lego/canonical_record> ?u .
//   FILTER(STRSTARTS(STR(?u), "http://identifiers.org/uniprot/"))
//   FILTER (isIRI(?u))
//   }
fn reacto_uniprot_rules(reacto_ont_graph: &FastGraph) -> Result<FastGraph, Box<dyn error::Error>> {
    let swrl_urn_ns = ns::Namespace::new("urn:swrl#")?;
    let swrl_urn_x = swrl_urn_ns.get("x")?;

    let swrl_ns = ns::Namespace::new("http://www.w3.org/2003/11/swrl#")?;

    let swrl_imp = swrl_ns.get("Imp")?;
    let swrl_variable = swrl_ns.get("Variable")?;
    let swrl_atom_list = swrl_ns.get("AtomList")?;
    let swrl_class_atom = swrl_ns.get("ClassAtom")?;
    let swrl_class_predicate = swrl_ns.get("classPredicate")?;
    let swrl_argument_1 = swrl_ns.get("argument1")?;
    let swrl_head = swrl_ns.get("head")?;
    let swrl_body = swrl_ns.get("body")?;

    let so_gene = term::StaticTerm::new_iri("http://purl.obolibrary.org/obo/SO_0000704")?;

    let mut output_graph: FastGraph = FastGraph::new();
    output_graph.insert(&swrl_urn_x, &ns::rdf::type_, &swrl_variable).unwrap();

    let geneontology_ns = ns::Namespace::new("http://geneontology.org/lego/")?;
    let geneontology_canonical_record = geneontology_ns.get("canonical_record")?;

    let mut count: i32 = 1;
    reacto_ont_graph
        .triples_with_p(&geneontology_canonical_record)
        .filter_triples(|t| t.o().value().to_string().starts_with("http://identifiers.org/uniprot/"))
        .for_each_triple(|t| {
            // this is already declared in purl.obolibrary.org/obo/go/snapshot/extensions/reacto.owl, which is an import in ontologies.ttl
            output_graph.insert(t.o(), &ns::rdf::type_, &ns::owl::Class).unwrap();
            output_graph.insert(t.o(), &ns::rdfs::subClassOf, &so_gene).unwrap();

            let id_2 = format!("reacto{:08}", count).to_string();
            let sub_bn_2: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_2).unwrap();
            count += 1;

            let id_3 = format!("reacto{:08}", count).to_string();
            let sub_bn_3: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_3).unwrap();
            count += 1;

            let id_4 = format!("reacto{:08}", count).to_string();
            let sub_bn_4: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_4).unwrap();
            count += 1;

            let id_5 = format!("reacto{:08}", count).to_string();
            let sub_bn_5: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_5).unwrap();
            count += 1;

            let id_6 = format!("reacto{:08}", count).to_string();
            let sub_bn_6: term::blank_node::BlankNode<String> = term::blank_node::BlankNode::new(id_6).unwrap();
            count += 1;

            output_graph.insert(&sub_bn_2, &ns::rdf::type_, &swrl_imp).unwrap();
            output_graph.insert(&sub_bn_2, &swrl_body, &sub_bn_3).unwrap();
            output_graph.insert(&sub_bn_3, &ns::rdf::type_, &swrl_atom_list).unwrap();
            output_graph.insert(&sub_bn_3, &ns::rdf::first, &sub_bn_4).unwrap();
            output_graph.insert(&sub_bn_4, &ns::rdf::type_, &swrl_class_atom).unwrap();
            output_graph.insert(&sub_bn_4, &swrl_class_predicate, t.s()).unwrap();
            output_graph.insert(&sub_bn_4, &swrl_argument_1, &swrl_urn_x).unwrap();
            output_graph.insert(&sub_bn_3, &ns::rdf::rest, &ns::rdf::nil).unwrap();
            output_graph.insert(&sub_bn_2, &swrl_head, &sub_bn_5).unwrap();
            output_graph.insert(&sub_bn_5, &ns::rdf::type_, &swrl_atom_list).unwrap();
            output_graph.insert(&sub_bn_5, &ns::rdf::first, &sub_bn_6).unwrap();
            output_graph.insert(&sub_bn_6, &ns::rdf::type_, &swrl_class_atom).unwrap();
            output_graph.insert(&sub_bn_6, &swrl_class_predicate, t.o()).unwrap();
            output_graph.insert(&sub_bn_6, &swrl_argument_1, &swrl_urn_x).unwrap();
            output_graph.insert(&sub_bn_5, &ns::rdf::rest, &ns::rdf::nil).unwrap();
        })
        .unwrap();

    Ok(output_graph)
}
