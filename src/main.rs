use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::env;
use std::process::exit;
use std::fs::File;
use std::rc::Rc;

#[macro_use] extern crate scan_fmt;

type Arcs = Vec<Rc<Arc>>;

struct Task {
    name: String,
    
    is_valid: bool,
}

struct Ressource {
    tasks: Vec<Task>,
    duration: u64,
    start_offset: u64,
    arc: Rc<Arc>
}

struct Arc {
    start_id: u64,
    end_id: u64,
    length: u64,
    capacity: u64,
    due_date: u64,
}

struct Node {
    id: u64,
    population: Option<u64>,
    max_rate: Option<u64>,
    incoming: Arcs,
    outgoing: Option<Rc<Arc>>,
}

struct Graph {
    safe_id: u64,
    nodes: BTreeMap<u64, Node>,
    routes: Vec<Arcs>,
}

impl Graph {
    fn new(safe_id: u64) -> Self {
        Graph {
            safe_id,
            nodes: BTreeMap::new(),
            routes: Vec::new()
        }
    }

    fn parse(r: impl BufRead) -> Result<Graph, ()> {
        let mut lines = r.lines().map(|l| l.expect("Could not read line"));
        lines.next(); // remove header
        let (evac_nb, safe_id) = scan_fmt!(&lines.next().unwrap(), "{} {}", u64, u64).unwrap();
        let mut graph = Graph::new(safe_id);
        let mut routes: Vec<Vec<_>> = Vec::new();

        for _ in 0..evac_nb {
            let line = lines.next().unwrap();
            let vals: Vec<_> = line.split(" ").map(|x| x.parse::<u64>().unwrap()).collect();
            let id = vals[0];
            let population = vals[1];
            let max_rate = vals[2];
            routes.push(vals[4..].to_vec());
            graph.add_node(Node{id, population: Some(population), max_rate: Some(max_rate), incoming: Vec::new(), outgoing: None});
        }

        lines.next(); // remove second header

        let line = lines.next().unwrap();
        let (_, arcs_nb) = scan_fmt!(&line, "{} {}", u64, u64).unwrap();
        for _ in 0..arcs_nb {
            let line = lines.next().unwrap();
            let (start_id, end_id, due_date, length, capacity) = scan_fmt!(&line, "{} {} {} {} {}", u64, u64, u64, u64, u64).unwrap();
            graph.add_arc(Rc::new(Arc{start_id, end_id, length, capacity, due_date}));
        }

        Ok(graph)
    }


    fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    fn add_arc(&mut self, arc: Rc<Arc>) {
        match self.nodes.get_mut(&arc.start_id) {
            Some(node_start) => {node_start.outgoing = Some(arc.clone());},
            None =>  {self.add_node(Node{id: arc.start_id, population: None, max_rate: None, incoming: Vec::new(), outgoing: Some(arc.clone())});}
        }

        match self.nodes.get_mut(&arc.end_id) {
            Some(node_end) => {node_end.incoming.push(arc)},
            None => {self.add_node(Node{id: arc.start_id, population: None, max_rate: None, incoming: vec!(arc), outgoing: None});}
        }
    }
    
    fn add_route(&mut self, route: Arcs) {
        self.routes.push(route);
    }

    fn write_to_file(&self) {

    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: metaheuristic <filename>");
        exit(1);
    }

    let filename = &args[1];
    let file = File::open(filename).expect(&format!("Could not open {}", filename));
    let file = BufReader::new(file);

    if let Ok(graph) = Graph::parse(file) {
        graph.write_to_file();
    } else {
        eprintln!("Could not parse graph");
        exit(1);
    }
}