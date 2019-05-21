use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, BufWriter};
use std::env;
use std::process::exit;
use std::fs::File;
use std::rc::Rc;
use std::io::Write;
use std::borrow::Cow;

use clap::{App, SubCommand};

#[macro_use] extern crate scan_fmt;

type Arcs = Vec<Rc<Arc>>;
type NodeId = u64;

struct GraphOut { nodes: Vec<NodeId>, edges: Vec<(NodeId, NodeId)> }

struct Task {
    node_id: u64,

    total_people: u64,

    evac_rate: u64,

    start_offset: u64,

    task_length: u64,

    is_valid: bool,

    /*
        A solution file only gives the three informations above
        So we need the original graph to evaluate the time_length of a task
    */

    // time_length = nb_of_people / evac_rate
    // time_length: u64
}

struct Resource {
    tasks: Vec<Task>,
    duration: u64,
    start_offset: u64,
    capacity: u64,
    arc: Rc<Arc>
}

impl Resource {

    fn add_task(&mut self, task:Task) -> bool {

        let task_length : u64;

        // A task lasts for total_people/evac_rate + eventually another unit of time if total people is not a multiple of evac_rate
        task_length = task.total_people / task.evac_rate + (if task.total_people % task.evac_rate > 0 {1} else {0});

        // Check for each unit of time if the capacity is not overloaded
        for t in task.start_offset..(task.start_offset+task_length+1) {

            let capacity_t: u64;

            if t < task.start_offset + task_length {
                capacity_t = task.total_people / task.evac_rate;
            }
            else {
                capacity_t = task.total_people % task.evac_rate;
            }

            for atask in self.tasks {

                if t >= atask.start_offset && t < atask.start_offset + atask.task_length {
                    capacity_t += atask.evac_rate;
                }
                else if t == atask.start_offset + atask.task_length {
                    if atask.total_people % atask.evac_rate > 0 {
                        capacity_t += atask.total_people % atask.evac_rate;
                    }
                    else {
                        capacity_t += atask.evac_rate;
                    }
                }

            }


            if capacity_t > self.capacity {
                return false
            }

        }

        
        self.tasks.append(task);
        return true;
    }

    fn check_capacity() -> bool {



        return true;

    }

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


    fn check_with_graph(&mut self, graph: Graph) -> bool {
        
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

        // Initialisation des ressources
        let resources : Vec<Resource>;

        // We can iterate directly on each route
        // The only important data in the solution is the start_offset at the beginning of the task

        for route in graph.routes {

            // the task corresponding to the route we are studying
            // For example we could check from which node the route begins
            let task : Task;

            let start_offset = task.start_offset;

            // Going through the route
            for arc in route {

                let the_res : Resource;

                if ()// resource corresponding to this arc doesn't already exist
                {

                    let new_res = Resource {

                        tasks : Vec::new(),
                        duration: arc.length,
                        start_offset: start_offset,
                        arc: arc
                        
                    };

                    // Add the resource
                    resources.append(new_res);

                    the_res = new_res;

                }

                else 
                {
                    // the_res = resources.find(Corresponding Resource according to the arc)
                }

                the_res.add_task(task);

                // If capacity is not sufficent, we stop checking the solution
                // and we return that it is invalid
                if (the_res.check_capacity() == FAIL){
                    self.is_valid = false;
                    return false; // Or a constant like SOLUTION_INVALID
                }

                // Else we continue

                start_offset = the_res.start_offset + the_res.duration;

            }

            
            


            resources.append(the_res)

        }


        // If we reach this line, the solution is_valid
        self.is_valid = true;
        return true;


        
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
    let matches = App::new("rust-mh")
        .version("1.0")
        .author("Lucien Menassol <lucien.menassol@gmail.com> Julien Bonnasserre")
        .about("Projet metaheuristique")
        .subcommand(SubCommand::with_name("solution")
                                      .about("Check the validity of a solution")
                                      .arg_from_usage("<file> 'Solution file'"))
        .subcommand(SubCommand::with_name("graph")
                                      .about("Render a graph to a dot file")
                                      .arg_from_usage("<in_file> 'In file'"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("solution") {
        let filename = matches.value_of("file").unwrap();
        let file = File::open(filename).unwrap();
        let file = BufReader::new(file);
        let sol = Solution::parse(file).expect("Invalid solution");
    }
    if let Some(matches) = matches.subcommand_matches("graph") {
        let filename = matches.value_of("in_file").unwrap();
        
        let file = File::open(filename).expect(&format!("Could not open {}", filename));
        let file = BufReader::new(file);
        let out_file = File::create("out.dot").expect(&format!("Could not create {}", "out.dot"));
        let mut out_file = BufWriter::new(out_file);

        if let Ok(graph) = Graph::parse(file) {
            graph.render_to(&mut out_file);
        } else {
            eprintln!("Could not parse graph");
            exit(1);
        }
    }
}