use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, BufWriter};
use std::env;
use std::process::exit;
use std::fs::File;
use std::rc::Rc;
use std::io::Write;
use std::borrow::Cow;

#[macro_use] extern crate scan_fmt;

type Arcs = Vec<Rc<Arc>>;
type NodeId = u64;

struct GraphOut { nodes: Vec<NodeId>, edges: Vec<(NodeId, NodeId)> }

struct Task {
    node_id: u64,

    evac_rate: u64,

    start_offset: u64,

    is_valid: bool,

    /*
        A solution file only gives the three informations above
        So we need the original graph to evaluate the time_length of a task
    */

    // time_length = nb_of_people / evac_rate
    // time_length: u64
}

struct Ressource {
    tasks: Vec<Task>,
    duration: u64,
    start_offset: u64,
    arc: Rc<Arc>
}

struct Arc {
    start_id: NodeId,
    end_id: NodeId,
    length: u64,
    capacity: u64,
    due_date: u64,
}

struct Node {
    id: NodeId,
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

    fn render_to(&self, w: &mut impl Write) {
        let mut edges = Vec::new();
        let mut nodes = Vec::new();
        for (k, n) in self.nodes.iter() {
            nodes.push(*k);
            for arc in &n.incoming {
                edges.push((arc.start_id, arc.end_id));
            }
        }

        let g = GraphOut{nodes, edges};
        dot::render(&g, w);
    }
}



/*

    SOLUTION_CHECKER :
    (opt)0. Parsing du fichier solution
    1. A l'ajout d'une tâche :
        1.a. L'ajouter aux ressources, dans l'ordre, pour tenir compte du décalage
            1.a.a. Pour chaque ressource, vérifier que la capacité n'est pas dépassé

    Ca donnait en python : 
    
    self.instance_name = instance_name
    self.evac_nb = evac_nb
    self.evac_nodes = evac_nodes
    self.is_valid = is_valid
    self.obj_func_value = obj_func_value
    self.eval_time = eval_time
    self.method_name = method_name
    self.comment = comment


*/

struct Solution {
    graph_name: String,
    nb_of_evac_nodes: u64,
    tasks: Vec<Task>,
    is_valid: bool,
    obj_value: u64,
    evaluation_time: u64,
    method: String,
    comment: String,
}

impl Solution {
    fn new() -> Self {
        Solution {
            graph_name : String::new(),
            nb_of_evac_nodes : 0,
            tasks : Vec::new(),
            is_valid : false,
            obj_value : 0,
            evaluation_time : 0,
            method : String::new(),
            comment : String::new(),
        }
    }

    fn parse(r: impl BufRead) -> Result<Solution, ()> {

        let mut sol = Solution::new();

        let mut lines = r.lines().map(|l| l.expect("Could not read line"));

        let graph_name = lines.next();
        sol.graph_name = graph_name.expect("");

        let nb_of_evac_nodes = scan_fmt!(&lines.next().unwrap(), "{}", u64).unwrap();
        sol.nb_of_evac_nodes = nb_of_evac_nodes;

        for _ in 0..nb_of_evac_nodes {

            let (node_id_to_evac, evac_rate, start_offset) = scan_fmt!(&lines.next().unwrap(), "{} {} {}", u64, u64, u64).unwrap();

            let task = Task{
                node_id: node_id_to_evac,

                // It looks like "is_valid" in a task is useless
                is_valid: false,

                evac_rate: evac_rate,
                start_offset: start_offset,
            };

            sol.add_task(task);

        }

        sol.is_valid = lines.next().unwrap().trim() == "valid";

        sol.obj_value = scan_fmt!(&lines.next().unwrap(), "{}", u64).unwrap();

        sol.evaluation_time = scan_fmt!(&lines.next().unwrap(), "{}", u64).unwrap();

        sol.method = lines.next().unwrap();

        sol.comment = lines.next().unwrap();

        Ok(sol)

    }
    
    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }


    fn check_with_graph(&mut self, graph: Graph) {
        
        // TODO
        /*
            + Generate an array of resources (one arc = one resource)

            + For each task in solution :
                (we'll add the task to each resource of its route while checking that capacity isn't overloaded)

                + Add the task to the first corresponding resource
                + For each resource in the evacuation task (in order of evac !) :

                    + Check in this resource if its capacity is respected
                    + If not overloaded :
                        + Add the task to the next resource with correct start_offset (depends on length of the resource)
                    + Else
                        sol.is_valid = false;
                        return is_valid;
            
            sol.is_valid = true;
            return is_valid;

        */
    }

}






impl<'a> dot::Labeller<'a, NodeId, (NodeId, NodeId)> for GraphOut {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("Graph").unwrap() }
    fn node_id(&'a self, n: &NodeId) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &NodeId) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(n.to_string().into())
    }
    fn edge_label<'b>(&'b self, _: &(NodeId, NodeId)) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr("&sube;".into())
    }
}

impl<'a> dot::GraphWalk<'a, NodeId, (NodeId, NodeId)> for GraphOut {
    fn nodes(&'a self) -> dot::Nodes<'a,NodeId> {Cow::Borrowed(&self.nodes)}
    fn edges(&'a self) ->  dot::Edges<'a, (NodeId, NodeId)> { Cow::Borrowed(&self.edges) }
    fn source(&self, e: &(NodeId, NodeId)) -> NodeId { e.0 }
    fn target(&self, e: &(NodeId, NodeId)) -> NodeId { e.1 }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: metaheuristic <filename>");
        exit(1);
    }

    let filename = &args[1];
    let out_filename = &args[2];
    let file = File::open(filename).expect(&format!("Could not open {}", filename));
    let file = BufReader::new(file);
    let out_file = File::create(out_filename).expect(&format!("Could not create {}", out_filename));
    let mut out_file = BufWriter::new(out_file);

    if let Ok(graph) = Graph::parse(file) {
        graph.render_to(&mut out_file);
    } else {
        eprintln!("Could not parse graph");
        exit(1);
    }
}