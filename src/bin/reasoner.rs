#[macro_use]
extern crate log;

use std::error;
use std::fs;
use std::io;
use std::io::{BufRead, BufWriter, Error, Write};
use std::ops::Deref;
use std::path;
use std::time;

use crepe::crepe;
use env_logger;
use humantime::format_duration;
use itertools::Itertools;
use structopt::StructOpt;

pub mod prp {
    use std::error;

    use crepe::crepe;

    crepe! {
        @input
        pub struct RDF<'a>(pub &'a str, pub &'a str, pub &'a str);

        @output
        pub struct Reachable<'a>(&'a str, &'a str, &'a str);

        @input
        pub struct Chain<'a>(&'a str, &'a str, &'a str, &'a str);

        @output
        pub struct Chainable<'a>(&'a str, &'a str, &'a str, &'a str);

        @output
        pub struct ListContains<'a>(&'a str, &'a str);

        // listContains(LIST, FIRST) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", FIRST).
        ListContains(list, first) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", first);

        // listContains(LIST, ITEM) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", OTHER), listContains(OTHER, ITEM).
        ListContains(list, item) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", other), ListContains(other, item);

        //prp-dom
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?p, "<http://www.w3.org/2000/01/rdf-schema#domain>", ?c), rdf(?x, ?p, _).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- RDF(p, "<http://www.w3.org/2000/01/rdf-schema#domain>", c), RDF(x, p, _);

        //prp-rng
        // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?p, "<http://www.w3.org/2000/01/rdf-schema#range>", ?c), rdf(_, ?p, ?y).
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- RDF(p, "<http://www.w3.org/2000/01/rdf-schema#range>", c), RDF(_, p, y);

        //prp-fp
        // rdf(?y1, "<http://www.w3.org/2002/07/owl#sameAs>", ?y2) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), rdf(?x, ?p, ?y1), rdf(?x, ?p, ?y2).
        Reachable(y1, "<http://www.w3.org/2002/07/owl#sameAs>", y2) <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), RDF(x, p, y1), RDF(x, p, y2);

        //prp-ifp
        // rdf(?x1, "<http://www.w3.org/2002/07/owl#sameAs>", ?x2) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#InverseFunctionalProperty>"), rdf(?x1, ?p, ?y), rdf(?x2, ?p, ?y).
        Reachable(x1, "<http://www.w3.org/2002/07/owl#sameAs>", x2) <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#FunctionalProperty>"), RDF(x1, p, y), RDF(x2, p, y);

        //prp-irp
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#IrreflexiveProperty>"), rdf(?x, ?p, ?x).
        // Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#IrreflexiveProperty>"), RDF(x, p, x);

        //prp-symp
        // rdf(?y, ?p, ?x) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#SymmetricProperty>"), rdf(?x, ?p, ?y).
        Reachable(y, p, x) <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#SymmetricProperty>"), RDF(x, p, y);

        //prp-asyp
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), rdf(?x, ?p, ?y), rdf(?y, ?p, ?x).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), RDF(x, p, y), RDF(y, p, x);
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AsymmetricProperty>"), RDF(x, p, y), RDF(y, p, x);

        //prp-trp
        // rdf(?x, ?p, ?z) :- rdf(?p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#TransitiveProperty>"), rdf(?x, ?p, ?y), rdf(?y, ?p, ?z).
        Reachable(x, p, z) <- RDF(p, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#TransitiveProperty>"), RDF(x, p, y), RDF(y, p, z);

        //prp-spo1
        // rdf(?x, ?p2, ?y) :- rdf(?p1, "<http://www.w3.org/2000/01/rdf-schema#subPropertyOf>", ?p2), rdf(?x, ?p1, ?y).
        Reachable(x, p2, y) <- RDF(p1, "<http://www.w3.org/2000/01/rdf-schema#subPropertyOf>", p2), RDF(x, p1, y);

        //prp-spo2
        //optimize by only creating chains after two hops?
        // chain(?s, ?p, ?o, ?y) :- rdf(?p, "<http://www.w3.org/2002/07/owl#propertyChainAxiom>", ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?s, ?p1, ?o), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
        Chainable(s, p, o, y) <- RDF(p, "<http://www.w3.org/2002/07/owl#propertyChainAxiom>", x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", p1), RDF(s, p1, o), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", y), (y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
        // chain(?s, ?p, ?o2, ?y) :- chain(?s, ?p, ?o, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?o, ?p1, ?o2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
        Chainable(s, p, o2, y) <- Chain(s, p, o, x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", p1), RDF(o, p1, o2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", y), (y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
        // rdf(?s, ?p, ?o2) :- chain(?s, ?p, ?o, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?p1), rdf(?o, ?p1, ?o2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>").
        Reachable(s, p, o2) <- Chain(s, p, o, x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", p1), RDF(o, p1, o2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");

        //prp-eqp1
        // rdf(?x, ?p2, ?y) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", ?p2), rdf(?x, ?p1, ?y).
        Reachable(x, p2, y) <- RDF(p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", p2), RDF(x, p1, y);

        //prp-eqp2
        // rdf(?x, ?p1, ?y) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", ?p2), rdf(?x, ?p2, ?y).
        Reachable(x, p1, y) <- RDF(p1, "<http://www.w3.org/2002/07/owl#equivalentProperty>", p2), RDF(x, p2, y);

        //prp-pdw
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
        // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?p1, "<http://www.w3.org/2002/07/owl#propertyDisjointWith>", ?p2), rdf(?x, ?p1, ?y), rdf(?x, ?p2, ?y).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(p1, "<http://www.w3.org/2002/07/owl#propertyDisjointWith>", p2), RDF(x, p1, y), RDF(x, p2, y);
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(p1, "<http://www.w3.org/2002/07/owl#propertyDisjointWith>", p2), RDF(x, p1, y), RDF(x, p2, y);

        //prp-adp
        // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
        // rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointProperties>"), rdf(?x, "<http://www.w3.org/2002/07/owl#members>", ?y), listContains(?y, ?pi), listContains(?y, ?pj), ?pi != ?pj, rdf(?u, ?pi, ?v), rdf(?u, ?pj, ?v).
        Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointProperties>"), RDF(x, "<http://www.w3.org/2002/07/owl#members>", y), ListContains(y, pi), ListContains(y, pj), (pi != pj), RDF(u, pi, v), RDF(u, pj, v);

        //prp-inv1
        //rdf(?y, ?p2, ?x) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#inverseOf>", ?p2), rdf(?x, ?p1, ?y).
        Reachable(y, p2, x) <- RDF(p1, "<http://www.w3.org/2002/07/owl#inverseOf>", p2), RDF(x, p1, y);

        //prp-inv2
        //rdf(?y, ?p1, ?x) :- rdf(?p1, "<http://www.w3.org/2002/07/owl#inverseOf>", ?p2), rdf(?x, ?p2, ?y).
        Reachable(y, p1, x) <- RDF(p1, "<http://www.w3.org/2002/07/owl#inverseOf>", p2), RDF(x, p2, y);

        //prp-key
        //TODO

        //prp-npa1
        // rdf(?i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>"),
        // rdf(?i2, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", ?i1), rdf(?x, "<http://www.w3.org/2002/07/owl#assertionProperty>", ?p), rdf(?x, "<http://www.w3.org/2002/07/owl#targetIndividual>", ?i2), rdf(?i1, ?p, ?i2).
        Reachable(i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", i1), RDF(x, "<http://www.w3.org/2002/07/owl#assertionProperty>", p), RDF(x, "<http://www.w3.org/2002/07/owl#targetIndividual>", i2), RDF(i1, p, i2);
        Reachable(i2, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", i1), RDF(x, "<http://www.w3.org/2002/07/owl#assertionProperty>", p), RDF(x, "<http://www.w3.org/2002/07/owl#targetIndividual>", i2), RDF(i1, p, i2);

        //prp-npa2
        // rdf(?i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", ?i1), rdf(?x, "<http://www.w3.org/2002/07/owl#assertionProperty>", ?p), rdf(?x, "<http://www.w3.org/2002/07/owl#targetValue>", ?lt), rdf(?i1, ?p, ?lt).
        Reachable(i1, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/2002/07/owl#sourceIndividual>", i1), RDF(x, "<http://www.w3.org/2002/07/owl#assertionProperty>", p), RDF(x, "<http://www.w3.org/2002/07/owl#targetValue>", lt), RDF(i1, p, lt);

    }

    pub fn run<'a>(data: &'a Vec<RDF<'_>>) -> Result<Vec<(&'a str, &'a str, &'a str)>, Box<dyn error::Error>> {
        let mut runtime = Crepe::new();
        runtime.extend(data);
        let (reachables, chainables, list_contains) = runtime.run();
        Ok(reachables.into_iter().map(|Reachable(a, b, c)| (a, b, c)).collect())
    }
}

pub mod cls {
    use std::error;

    use crepe::crepe;

    crepe! {
        @input
        pub struct RDF<'a>(pub &'a str, pub &'a str, pub &'a str);

        @output
        pub struct Reachable<'a>(&'a str, &'a str, &'a str);

        @output
        pub struct ListContains<'a>(&'a str, &'a str);

        // listContains(LIST, FIRST) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", FIRST).
        ListContains(list, first) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", first);

        // listContains(LIST, ITEM) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", OTHER), listContains(OTHER, ITEM).
        ListContains(list, item) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", other), ListContains(other, item);

        //cls-int1
        //optimize by only creating ichains after two hops?
        // ichain(?i, ?c, ?y) :- rdf(?c, "<http://www.w3.org/2002/07/owl#intersectionOf>", ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
        Reachable(i, c, y) <- RDF(c, "<http://www.w3.org/2002/07/owl#intersectionOf>", x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", c1), RDF(i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", y), (y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
        // ichain(?i, ?c, ?y) :- ichain(?i, ?c, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", ?y), ?y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".
        Reachable(i, c, y) <- RDF(i, c, x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", c1), RDF(i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", y), (y != "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
        // rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- ichain(?i, ?c, ?x), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", ?c1), rdf(?i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>").
        Reachable(i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- RDF(i, c, x), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", c1), RDF(i, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");

        //cls-int2
        // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1) :- rdf(?c, "<http://www.w3.org/2002/07/owl#intersectionOf>", ?x), listContains(?x, ?c1), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c).
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1) <- RDF(c, "<http://www.w3.org/2002/07/owl#intersectionOf>", x), ListContains(x, c1), RDF(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c);

        //cls-uni
        // rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c) :- rdf(?c, "<http://www.w3.org/2002/07/owl#unionOf>", ?x), listContains(?x, ?c1), rdf(?y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- RDF(c, "<http://www.w3.org/2002/07/owl#unionOf>", x), ListContains(x, c1), RDF(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1);

        //cls-com
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?c1, "<http://www.w3.org/2002/07/owl#complementOf>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(c1, "<http://www.w3.org/2002/07/owl#complementOf>", c2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2);

        //cls-svf1
        // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, ?v), rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?y).
        Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x) <- RDF(x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", y), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, p, v), RDF(v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", y);

        //cls-svf2
        // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", "<http://www.w3.org/2002/07/owl#Thing>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, _).
        Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x) <- RDF(x, "<http://www.w3.org/2002/07/owl#someValuesFrom>", "<http://www.w3.org/2002/07/owl#Thing>"), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, p, _);

        //cls-avf
        // rdf(?v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?y) :- rdf(?x, "<http://www.w3.org/2002/07/owl#allValuesFrom>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, ?v).
        Reachable(v, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", y) <- RDF(x, "<http://www.w3.org/2002/07/owl#allValuesFrom>", y), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x), RDF(u, p, v);

        //cls-hv1
        // rdf(?u, ?p, ?y) :- rdf(?x, "<http://www.w3.org/2002/07/owl#hasValue>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x).
        Reachable(u, p, y) <- RDF(x, "<http://www.w3.org/2002/07/owl#hasValue>", y), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x);

        //cls-hv2
        // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x) :- rdf(?x, "<http://www.w3.org/2002/07/owl#hasValue>", ?y), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, ?p, ?y).
        Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x) <- RDF(x, "<http://www.w3.org/2002/07/owl#hasValue>", y), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, p, y);

        //cls-maxc1
        // rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"0\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, _).
        Reachable(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"0\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x), RDF(u, p, _);

        //cls-maxc2
        // rdf(?y1, "<http://www.w3.org/2002/07/owl#sameAs>", ?y2) :- rdf(?x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"1\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), rdf(?x, "<http://www.w3.org/2002/07/owl#onProperty>", ?p), rdf(?u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?x), rdf(?u, ?p, ?y1), rdf(?u, ?p, ?y2).
        Reachable(y1, "<http://www.w3.org/2002/07/owl#sameAs>", y2) <- RDF(x, "<http://www.w3.org/2002/07/owl#maxCardinality>", "\"1\"^^<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>"), RDF(x, "<http://www.w3.org/2002/07/owl#onProperty>", p), RDF(u, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", x), RDF(u, p, y1), RDF(u, p, y2);

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
        Reachable(y, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c) <- RDF(c, "<http://www.w3.org/2002/07/owl#oneOf>", x), ListContains(x, y);

    }

    pub fn run<'a>(data: &'a Vec<RDF<'_>>) -> Result<Vec<(&'a str, &'a str, &'a str)>, Box<dyn error::Error>> {
        let mut runtime = Crepe::new();
        runtime.extend(data);
        let (reachables, list_contains) = runtime.run();
        Ok(reachables.into_iter().map(|Reachable(a, b, c)| (a, b, c)).collect())
    }
}

pub mod cax {
    use std::error;

    use crepe::crepe;

    crepe! {
        @input
        pub struct RDF<'a>(pub &'a str, pub &'a str, pub &'a str);

        @output
        pub struct Reachable<'a>(&'a str, &'a str, &'a str);

        @output
        pub struct ListContains<'a>(&'a str, &'a str);

        // listContains(LIST, FIRST) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", FIRST).
        ListContains(list, first) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>", first);

        // listContains(LIST, ITEM) :- rdf(LIST, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", OTHER), listContains(OTHER, ITEM).
        ListContains(list, item) <- RDF(list, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>", other), ListContains(other, item);

        //cax-sco
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2) :- rdf(?c1, "<http://www.w3.org/2000/01/rdf-schema#subClassOf?", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2) <- RDF(c1, "<http://www.w3.org/2000/01/rdf-schema#subClassOf?", c2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1);

        //cax-eqc1
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2) :- rdf(?c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2) <- RDF(c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", c2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1);

        //cax-eqc2
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1) :- rdf(?c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1) <- RDF(c1, "<http://www.w3.org/2002/07/owl#equivalentClass>", c2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2);

        //cax-dw
        // rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?c1, "<http://www.w3.org/2002/07/owl#disjointWith>", ?c2), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c1), rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?c2).
        Reachable(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(c1, "<http://www.w3.org/2002/07/owl#disjointWith>", c2), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c1), RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", c2);

        //cax-adc
        // rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") :- rdf(?x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointClasses>"), rdf(?x, "<http://www.w3.org/2002/07/owl#members>", ?y), listContains(?y, ?ci), listContains(?y, ?cj), ?ci != ?cj, rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?ci), rdf(?z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ?cj).
        Reachable(z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#Nothing>") <- RDF(x, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://www.w3.org/2002/07/owl#AllDisjointClasses>"), RDF(x, "<http://www.w3.org/2002/07/owl#members>", y), ListContains(y, ci), ListContains(y, cj), (ci != cj), RDF(z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", ci), RDF(z, "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", cj);

    }

    pub fn run<'a>(data: &'a Vec<RDF<'_>>) -> Result<Vec<(&'a str, &'a str, &'a str)>, Box<dyn error::Error>> {
        let mut runtime = Crepe::new();
        runtime.extend(data);
        let (reachables, list_contains) = runtime.run();
        Ok(reachables.into_iter().map(|Reachable(a, b, c)| (a, b, c)).collect())
    }
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

    let input = fs::read_to_string(&options.input)?;

    debug!("reading raw data");
    let raw_data = input
        .lines()
        .into_iter()
        .map(|line| line.trim_end_matches("."))
        .map(|line| {
            let asdf: Vec<&str> = line.split(' ').collect();
            asdf
        })
        .collect_vec();

    // let data: Vec<prp::RDF<'_>> = input
    //     .lines()
    //     .into_iter()
    //     .map(|line| line.trim_end_matches("."))
    //     .map(|line| line.split(' ').collect())
    //     .map(|split: Vec<&str>| prp::RDF(split[0], split[1], split[2]))
    //     .collect_vec();

    // debug!("processing prp rules");
    // let prp_output: path::PathBuf = options.output.clone().join("reasoned-prp.nt");
    // let mut prp_bw = io::BufWriter::new(fs::File::create(&prp_output)?);
    // let prp_data = raw_data.iter().map(|split| prp::RDF(split[0], split[1], split[2])).collect_vec();
    // let prp_results = prp::run(&prp_data)?;
    // for (x, y, z) in prp_results.into_iter() {
    //     prp_bw.write_all(format!("{} {} {} .", x, y, z).as_bytes())?;
    // }

    // debug!("processing cls rules");
    // let cls_output: path::PathBuf = options.output.clone().join("reasoned-cls.nt");
    // let mut cls_bw = io::BufWriter::new(fs::File::create(&cls_output)?);
    // let cls_data = raw_data.iter().map(|split| cls::RDF(split[0], split[1], split[2])).collect_vec();
    // let cls_results = cls::run(&cls_data)?;
    // for (x, y, z) in cls_results.into_iter() {
    //     cls_bw.write_all(format!("{} {} {} .", x, y, z).as_bytes())?;
    // }

    debug!("processing cax rules");
    let cax_output: path::PathBuf = options.output.clone().join("reasoned-cax.nt");
    let mut cax_bw = io::BufWriter::new(fs::File::create(&cax_output)?);
    let cax_data = raw_data.iter().map(|split| cax::RDF(split[0], split[1], split[2])).collect_vec();
    let cax_results = cax::run(&cax_data)?;
    for (x, y, z) in cax_results.into_iter() {
        cax_bw.write_all(format!("{} {} {} .", x, y, z).as_bytes())?;
    }

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
