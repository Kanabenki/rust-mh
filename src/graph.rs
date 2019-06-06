use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};

use std::io::Write;
use std::io::BufRead;
use std::rc::Rc;

type Arcs = Vec<Rc<Arc>>;
type NodeId = u64;

struct GraphOut {
    nodes: Vec<NodeId>,
    edges: Vec<(NodeId, NodeId)>,
}

#[derive(Clone, Debug)]
struct Task {
    node_id_to_evac: u64,

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

impl Task {
    fn new(
        node_id_to_evac: u64,
        total_people: u64,
        evac_rate: u64,
        start_offset: u64,
        is_valid: bool,
    ) -> Self {
        // A task lasts for total_people/evac_rate + eventually another unit of time if total people is not a multiple of evac_rate
        let task_length =
            total_people / evac_rate + (if (total_people % evac_rate) > 0 { 1 } else { 0 });

        Task {
            node_id_to_evac,
            total_people,
            evac_rate,
            start_offset,
            task_length,
            is_valid,
        }
    }
}

enum AddTaskResult {
    Ok,
    Failed(u64)
}

/*
    A resource is defined :
    - its duration (time to travel through the resource)
    - a start offset
    - a capacity (that can't be exceeded at each t)

    - a list of tasks involved

    This structure doesn't take into account the objective function (the supposed end of evacuation)

*/
struct Resource {
    tasks: Vec<Task>,
    duration: u64,
    arc: Rc<Arc>,
}

impl Resource {
    // Add a task to this resource >IF< it doesn't overload the resource
    fn add_task(&mut self, task: Task) -> AddTaskResult {
        use AddTaskResult::{Failed, Ok};
        let task_length = task.task_length;

        // Check for each unit of time if the capacity is not overloaded
        for t in task.start_offset..(task.start_offset + task_length) {
            // Capacity needed by the added task
            let mut capacity_t = if t < task.start_offset + task_length - 1 {
                task.evac_rate
            } else {
                task.total_people % task.evac_rate
            };

            // ... adding the capacity of all the other tasks running at the same t
            for atask in &self.tasks {
                if t >= atask.start_offset && t < (atask.start_offset + atask.task_length - 1) {
                    capacity_t += atask.evac_rate;
                } else if t == atask.start_offset + atask.task_length - 1 {
                    if (atask.total_people % atask.evac_rate) > 0 {
                        capacity_t += atask.total_people % atask.evac_rate;
                    } else {
                        capacity_t += atask.evac_rate;
                    }
                }
            }

            // Stops and return false if the capacity is overloaded
            // (the task won't be added to the resource)
            if capacity_t > self.arc.as_ref().capacity {
                let max = self.tasks.iter().fold(0, |max, task| std::cmp::max(max, task.start_offset + task.task_length));
                return Failed(max);
            }
        }

        // Adds task to the resource (since it doesn't overload the resource)
        self.tasks.push(task);
        return Ok;
    }

    fn check_capacity() -> bool {
        return true;
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Arc {
    start_id: NodeId,
    end_id: NodeId,
    length: u64,
    capacity: u64,
    due_date: u64,
}

#[derive(Debug)]
struct Node {
    id: NodeId,
    population: Option<u64>,
    max_rate: Option<u64>,
    incoming: Arcs,
    outgoing: Option<Rc<Arc>>,
}

#[derive(Debug)]
pub struct Graph {
    safe_id: u64,
    nodes: BTreeMap<u64, Node>,
    routes: Vec<Arcs>,
}

impl Graph {
    pub fn new(safe_id: u64) -> Self {
        Graph {
            safe_id,
            nodes: BTreeMap::new(),
            routes: Vec::new(),
        }
    }

    pub fn parse(r: impl BufRead) -> Result<Graph, ()> {
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
            let mut route = Vec::new();
            route.push((vals[0], vals[4]));
            for i in 4..(4 + vals[4..].len() - 1) {
                route.push((vals[i], vals[i + 1]));
            }
            routes.push(route);
            graph.add_node(Node {
                id,
                population: Some(population),
                max_rate: Some(max_rate),
                incoming: Vec::new(),
                outgoing: None,
            });
        }

        lines.next(); // remove second header

        let line = lines.next().unwrap();
        let (_, arcs_nb) = scan_fmt!(&line, "{} {}", u64, u64).unwrap();
        for _ in 0..arcs_nb {
            let line = lines.next().unwrap();
            let (s_id, e_id, due_date, length, capacity) =
                scan_fmt!(&line, "{} {} {} {} {}", u64, u64, u64, u64, u64).unwrap();
            for route in &routes {
                for (start_r, end_r) in route {
                    if s_id == *start_r && e_id == *end_r {
                        graph.add_arc(Rc::new(Arc {
                            start_id: s_id,
                            end_id: e_id,
                            length,
                            capacity,
                            due_date,
                        }));
                    } else if s_id == *end_r && e_id == *start_r {
                        graph.add_arc(Rc::new(Arc {
                            start_id: e_id,
                            end_id: s_id,
                            length,
                            capacity,
                            due_date,
                        }));
                    }
                }
            }
        }

        graph.routes = routes
            .iter()
            .map(|route| {
                route
                    .iter()
                    .map(|(start_id, _)| {
                        graph
                            .nodes
                            .get(start_id)
                            .expect("Missing node")
                            .outgoing
                            .as_ref()
                            .expect(&format!("Missing outgoing for {}", start_id))
                            .clone()
                    })
                    .collect()
            })
            .collect();

        Ok(graph)
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    fn add_arc(&mut self, arc: Rc<Arc>) {
        match self.nodes.get_mut(&arc.start_id) {
            Some(ref mut node_start) if node_start.outgoing.is_none() => {
                node_start.outgoing = Some(arc.clone());
            }
            None => {
                self.add_node(Node {
                    id: arc.start_id,
                    population: None,
                    max_rate: None,
                    incoming: Vec::new(),
                    outgoing: Some(arc.clone()),
                });
            }
            _ => {}
        }

        match self.nodes.get_mut(&arc.end_id) {
            Some(ref mut node_end) if !node_end.incoming.contains(&arc) => {
                node_end.incoming.push(arc)
            }
            None => {
                self.add_node(Node {
                    id: arc.end_id,
                    population: None,
                    max_rate: None,
                    incoming: vec![arc],
                    outgoing: None,
                });
            }
            _ => (),
        }
    }

    fn add_route(&mut self, route: Arcs) {
        self.routes.push(route);
    }

    pub fn render_to(&self, w: &mut impl Write) {
        let mut edges = Vec::new();
        let mut nodes = Vec::new();
        for (k, n) in self.nodes.iter() {
            nodes.push(*k);
            for arc in &n.incoming {
                edges.push((arc.start_id, arc.end_id));
            }
        }

        let g = GraphOut { nodes, edges };
        dot::render(&g, w);
    }

    pub fn get_bounds(&self) -> (u64, u64) {
        let times: Vec<_> = self.routes.iter().map(|route| {
            let pop = self.nodes.get(&route[0].start_id).expect("Missing node").population.expect("No population");
            let length = route.iter().fold(0, |tot, arc|  tot + arc.length);
            let min_cap = route.iter().fold(std::u64::MAX, |min, arc| std::cmp::min(min, arc.capacity));
            pop / min_cap + (if pop % min_cap == 0 {0} else {1}) + length - 1
        }).collect();

        (*times.iter().max().unwrap(), times.iter().sum())
    }

    pub fn generate_solution(&self) -> Solution {
        use AddTaskResult::*;

        let mut resources: Vec<Resource> = Vec::new();
        let mut arc_set = HashSet::new();


        for route in &self.routes {
            let id = route[0].as_ref().start_id;
            let mut task = Task::new(id, self.nodes.get(&id).expect("Missing node").population.expect("No pop"), 0, 0, true);
            for arc in route {

                let the_res = if !arc_set.contains(arc.as_ref()) {
                    arc_set.insert(arc.as_ref());

                    let new_res = Resource {
                        tasks: Vec::new(),
                        duration: arc.length,
                        arc: arc.clone(),
                    };

                    resources.push(new_res);
                    resources.last_mut().unwrap()
                } else {
                    resources
                        .iter_mut()
                        .find(|res| res.arc.as_ref() == arc.as_ref())
                        .unwrap()
                };

                match the_res.add_task(task.clone()) {
                    Ok => {},
                    Failed(max) => {
                        
                    },
                }

                task.start_offset += the_res.duration;
            }
        }

        unimplemented!();
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

pub struct Solution {
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
    pub fn new() -> Self {
        Solution {
            graph_name: String::new(),
            nb_of_evac_nodes: 0,
            tasks: Vec::new(),
            is_valid: false,
            obj_value: 0,
            evaluation_time: 0,
            method: String::new(),
            comment: String::new(),
        }
    }

    pub fn parse(r: impl BufRead, graph: &Graph) -> Result<Solution, ()> {
        let mut sol = Solution::new();

        let mut lines = r.lines().map(|l| l.expect("Could not read line"));

        let graph_name = lines.next();
        sol.graph_name = graph_name.expect("");

        let nb_of_evac_nodes = scan_fmt!(&lines.next().unwrap(), "{}", u64).unwrap();
        sol.nb_of_evac_nodes = nb_of_evac_nodes;

        for _ in 0..nb_of_evac_nodes {
            let (node_id_to_evac, evac_rate, start_offset) =
                scan_fmt!(&lines.next().unwrap(), "{} {} {}", u64, u64, u64).unwrap();

            let task = Task::new(node_id_to_evac, graph.nodes.get(&node_id_to_evac).expect("Missing node for task").population.expect("Node has no people"), evac_rate, start_offset, false);
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

    pub fn check_with_graph(&mut self, graph: &Graph) -> bool {
        use AddTaskResult::Failed;
        //for route in &graph.routes {
        //    for arc in &route {
        //        let
        //    }
        //}

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
        let mut resources: Vec<Resource> = Vec::new();
        let mut arc_set = HashSet::new();

        // We can iterate directly on each route
        // The only important data in the solution is the start_offset at the beginning of the task
        for route in &graph.routes {
            // the task corresponding to the route we are studying
            // //!!!\\ SO WE HAVE TO ACTUALLY FIND THE TASK, CORRESPONDING TO THE ROUTE, IN THE SOLUTION
            // For example we could check from which node the route begins
            let id = route[0].as_ref().start_id;
            let mut task = self.tasks.iter().find(|task| task.node_id_to_evac == id).unwrap().clone();
            // Going through the route // WILL IT ITERATE OVER IT IN THE GOOD ORDER?
            for arc in route {

                // resource corresponding to this arc doesn't already exist
                let the_res = if !arc_set.contains(arc.as_ref()) {
                    arc_set.insert(arc.as_ref());

                    let new_res = Resource {
                        tasks: Vec::new(),
                        duration: arc.length,
                        arc: arc.clone(),
                    };

                    // Add the resource
                    resources.push(new_res);
                    resources.last_mut().unwrap()
                } else {
                    resources
                        .iter_mut()
                        .find(|res| res.arc.as_ref() == arc.as_ref())
                        .unwrap()
                };

                let is_task_valid = the_res.add_task(task.clone());

                // If capacity is not sufficent, we stop checking the solution
                // and we return that it is invalid
                if let Failed(_) = is_task_valid {
                    println!("Validation failed on arc {:?} with task {:?}", arc, task);
                    self.is_valid = false;
                    return false; // Or a constant like SOLUTION_INVALID
                }
                // Else we continue
                task.start_offset += the_res.duration;
            }
        }

        // If we reach this line, the solution is_valid
        self.is_valid = true;
        return true;
    }
}

/* Creating a solution :
    The teacher advised us to use a greedy algorithm :
    -> Just start from any node (alphetical order? numerical?)
    -> Add task as it is if possible, or put an offset if not


*/
impl<'a> dot::Labeller<'a, NodeId, (NodeId, NodeId)> for GraphOut {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("Graph").unwrap()
    }
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
    fn nodes(&'a self) -> dot::Nodes<'a, NodeId> {
        Cow::Borrowed(&self.nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a, (NodeId, NodeId)> {
        Cow::Borrowed(&self.edges)
    }
    fn source(&self, e: &(NodeId, NodeId)) -> NodeId {
        e.0
    }
    fn target(&self, e: &(NodeId, NodeId)) -> NodeId {
        e.1
    }
}