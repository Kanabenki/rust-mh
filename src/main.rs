mod graph;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::process::exit;
use graph::*;

use clap::{App, SubCommand};

#[macro_use]
extern crate scan_fmt;

fn main() {
    let matches = App::new("rust-mh")
        .version("1.0")
        .author("Lucien Menassol <lucien.menassol@gmail.com> Julien Bonnasserre")
        .about("Projet metaheuristique")
        .args_from_usage(
            "-p --print-graph          'Prints the parsed graph'
             -s, --solution=[solution] 'Validate a solution file'
             <graph>                   'Input graph file'",
        )
        .get_matches();

    let filename = matches.value_of("graph").unwrap();
    let file = File::open(filename).expect(&format!("Could not open {}", filename));
    let file = BufReader::new(file);

    println!("Checking graph");
    if let Ok(graph) = Graph::parse(file) {
        println!("Graph is valid");
        if matches.is_present("print-graph") {
            println!("{:#?}", graph);
        }
        let out_file = File::create("out.dot").expect(&format!("Could not create {}", "out.dot"));
        let mut out_file = BufWriter::new(out_file);
        graph.render_to(&mut out_file);
        println!("Graph rendered to out.dot");

        if let Some(filename) = matches.value_of("solution") {
            println!("Checking solution");
            let sol_file = File::open(filename).expect(&format!("Could not open {}", filename));
            let sol_file = BufReader::new(sol_file);
            let mut sol = Solution::parse(sol_file, &graph).expect("Could not parse solution");
            match sol.check_with_graph(&graph) {
                false => println!("Invalid solution"),
                true => println!("Valid solution"),
            }
        }
    } else {
        eprintln!("Could not parse graph");
        exit(1);
    }
}
