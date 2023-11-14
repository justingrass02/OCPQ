use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::Map,
    time::Instant,
};

use axum::{extract::State, http::StatusCode, Json};
use itertools::Itertools;
use pm_rust::event_log::ocel::ocel_struct::{OCELEvent, OCELObject, OCEL};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{with_ocel_from_state, AppState};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
enum DependencyType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "existsInTarget")]
    ExistsInTarget,
    #[serde(rename = "existsInSource")]
    ExistsInSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NodeDependency {
    #[serde(rename = "sourceQualifier")]
    source_qualifier: String,
    #[serde(rename = "targetQualifier")]
    target_qualifier: String,
    #[serde(rename = "objectType")]
    object_type: String,
    #[serde(rename = "dependencyType")]
    dependency_type: DependencyType,
    #[serde(rename = "variableName")]
    variable_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNode {
    #[serde(rename = "eventType")]
    event_type: String,
    parents: Vec<TreeNodeDependency>,
    children: Vec<TreeNodeDependency>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TreeNodeDependency {
    dependency: NodeDependency,
    #[serde(rename = "eventType")]
    event_type: String,
}

#[derive(Debug)]
pub enum BoundValue {
    Single(String),
    Multiple(Vec<String>),
}

struct LinkedOCEL<'a> {
    pub event_map: HashMap<String, &'a OCELEvent>,
    pub object_map: HashMap<String, &'a OCELObject>,
    pub events_of_type: HashMap<String, Vec<&'a OCELEvent>>,
    pub objects_of_type: HashMap<String, Vec<&'a OCELObject>>,
}

fn link_ocel_info(ocel: &OCEL) -> LinkedOCEL {
    let event_map: HashMap<String, &OCELEvent> = ocel
        .events
        .par_iter()
        .map(|ev| (ev.id.clone(), ev))
        .collect();
    let object_map: HashMap<String, &OCELObject> = ocel
        .objects
        .par_iter()
        .map(|obj| (obj.id.clone(), obj))
        .collect();

    let events_of_type: HashMap<String, Vec<&OCELEvent>> = ocel
        .event_types
        .par_iter()
        .map(|ev_type| {
            (
                ev_type.name.clone(),
                ocel.events
                    .iter()
                    .filter(|ev| ev.event_type == ev_type.name)
                    .map(|ev| ev)
                    .collect(),
            )
        })
        .collect();

    let objects_of_type: HashMap<String, Vec<&OCELObject>> = ocel
        .event_types
        .par_iter()
        .map(|obj_type| {
            (
                obj_type.name.clone(),
                ocel.objects
                    .iter()
                    .filter(|ev| ev.object_type == obj_type.name)
                    .map(|ev| ev)
                    .collect(),
            )
        })
        .collect();
    LinkedOCEL {
        event_map: event_map,
        object_map: object_map,
        events_of_type: events_of_type,
        objects_of_type: objects_of_type,
    }
}

fn get_new_bound_variables_from_node(
    node: &TreeNode,
    events_of_type: &HashMap<String, Vec<&OCELEvent>>,
    prev_bound_vars: &HashSet<&String>,
) -> Vec<(AdditionalBindingInfo, HashMap<String, Vec<BoundValue>>)> {
    if node.children.is_empty() {
        // No variables to add
        return Vec::new();
    }
    // Get all events of the corresponding event type (determined by node)
    let events = events_of_type.get(&node.event_type).unwrap();
    // Required Bindings: Variable that require binding (i.e., were not already bound before)
    let required_bindings: HashSet<(String, String, DependencyType)> = node
        .children
        .clone()
        .into_iter()
        .filter(|c| !prev_bound_vars.contains(&c.dependency.variable_name))
        .map(|c| {
            (
                c.dependency.source_qualifier,
                c.dependency.variable_name,
                c.dependency.dependency_type,
            )
        })
        .collect();
    let events_bindings: Vec<(AdditionalBindingInfo, HashMap<String, Vec<BoundValue>>)> = events
        .into_par_iter()
        .filter_map(|ev| match &ev.relationships {
            Some(rels) => {
                let mut binding_values: HashMap<String, Vec<BoundValue>> = required_bindings
                    .iter()
                    .map(|(_, variable_name, _)| (variable_name.clone(), Vec::new()))
                    .collect();
                for (qualifier, variable_name, dependency_type) in &required_bindings {
                    let object_id_vals: Vec<String> = rels
                        .iter()
                        .filter(|rel| rel.qualifier == *qualifier)
                        .map(|rel| rel.object_id.clone())
                        .collect();
                    match dependency_type {
                        DependencyType::Simple => {
                            binding_values.get_mut(variable_name).unwrap().extend(
                                object_id_vals.into_iter().map(|id| BoundValue::Single(id)),
                            );
                        }
                        DependencyType::All => binding_values
                            .get_mut(variable_name)
                            .unwrap()
                            .push(BoundValue::Multiple(object_id_vals)),
                        DependencyType::ExistsInSource => binding_values
                            .get_mut(variable_name)
                            .unwrap()
                            .push(BoundValue::Multiple(object_id_vals)),
                        DependencyType::ExistsInTarget => {
                            binding_values.get_mut(variable_name).unwrap().extend(
                                object_id_vals.into_iter().map(|id| BoundValue::Single(id)),
                            );
                        }
                    }
                }
                return Some((
                    AdditionalBindingInfo {
                        past_events: vec![ev.id.clone()],
                    },
                    binding_values,
                ));
            }
            None => {
                eprintln!("Node has child dependency but event has no linked objects");
                return None;
            }
        })
        .collect();
    return events_bindings;
}

#[derive(Clone, Debug)]
pub struct AdditionalBindingInfo {
    past_events: Vec<String>,
}
fn combine_bindings<'a>(
    prev_binding: Option<(&'a AdditionalBindingInfo,&'a HashMap<&'a String, &'a BoundValue>)>,
    new_vars_with_value: &'a Vec<(AdditionalBindingInfo, HashMap<String, Vec<BoundValue>>)>,
) -> Vec<(AdditionalBindingInfo, HashMap<&'a String, &'a BoundValue>)> {
    let new_bindings: Vec<(AdditionalBindingInfo, HashMap<&String, &BoundValue>)> =
        new_vars_with_value
            .par_iter()
            .flat_map(|(info, b)| {
                if b.len() > 0 {
                    let cat = b
                        .into_iter()
                        .filter(|(_, b)| b.len() > 0)
                        .map(|(a, bs)| bs.into_iter().map(move |b| (a, b)).collect_vec())
                        .multi_cartesian_product();
                    let xxx: Vec<(AdditionalBindingInfo, HashMap<&String, &BoundValue>)> = match prev_binding {
                        Some((prev_info,prev_b)) => cat
                            .cartesian_product(prev_b.into_iter())
                            .flat_map(|(new, (x,y))| {
                                vec![new,vec![(*x, *y)]]
                            })
                            .map(|x| (info.clone(), x.into_iter().collect()))
                            .collect(),
                        None => cat
                            .map(|x| (info.clone(), x.into_iter().collect()))
                            .collect(),
                    };
                    if prev_binding.is_some() {
                        println!("XXX {:?}",xxx);
                    }
                    return xxx;
                } else {
                    match prev_binding {
                        Some((prev_info,prev_b)) => {
                            let xxx : Vec<(AdditionalBindingInfo, HashMap<&String, &BoundValue>)>  = vec![(prev_info.clone(), prev_b.clone())];
                            return xxx;
                        },
                        None => {
                            Vec::new()
                        },
                    }
                }
            })
            .collect();

    return new_bindings;
}
type Binding<'a> = Vec<(AdditionalBindingInfo, HashMap<&'a String, &'a BoundValue>)>;
pub fn get_sat_bindings<'a>(
    node: &TreeNode,
    binding: &'a Binding<'a>,
    events_of_type: &HashMap<String, Vec<&OCELEvent>>,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> Vec<&'a (AdditionalBindingInfo, HashMap<&'a String, &'a BoundValue>)> {
    let sat_bindings: Vec<&(AdditionalBindingInfo, HashMap<&String, &BoundValue>)> = binding
        .par_iter()
        .filter(|(binding_info, binding)| {
            let c_node = node.clone();
            let next_evs = events_of_type.get(&c_node.event_type).unwrap();
            next_evs.iter().any(|next_event| {
                c_node.clone().parents.iter().all(|p| {
                    match_dependency(
                        &p.dependency,
                        binding,
                        binding_info,
                        &next_event.id,
                        &event_map,
                        &object_map,
                    )
                })
            })
        })
        .collect();
    sat_bindings
}

pub fn check_with_tree(nodes: Vec<TreeNode>, ocel: &OCEL) -> Vec<String> {
    let LinkedOCEL {
        event_map,
        object_map,
        events_of_type,
        objects_of_type,
    } = link_ocel_info(ocel);

    if nodes.len() > 0 {
        let current_node = nodes[0].clone();
        let evs_bindings: Vec<(AdditionalBindingInfo, HashMap<String, Vec<BoundValue>>)> =
            get_new_bound_variables_from_node(&current_node, &events_of_type, &HashSet::new());

        println!("Required bindings: {}", evs_bindings.len());
        let binding: Binding = combine_bindings(None, &evs_bindings);
        println!("Total Combinations: {}", binding.len());
        println!("First combination: {:#?}", binding[0]);
        println!("Nodes len: {}", nodes.len());
        if nodes.len() >= 3 {
            let next_node = nodes[1].clone();
            let next_next_node = nodes[2].clone();
            // let required_vars = next_node
            //     .parents.clone()
            //     .into_iter()
            //     .map(|p| p.dependency.variable_name)
            //     .collect_vec();
            // println!("Required vars for second node: {:#?}", required_vars);
            let sat_bindings: Vec<&(AdditionalBindingInfo, HashMap<&String, &BoundValue>)> =
                get_sat_bindings(
                    &next_node,
                    &binding,
                    &events_of_type,
                    &event_map,
                    &object_map,
                );
            println!("After sat-bindings (#: {})", sat_bindings.len());
            // let sat_bindings: Vec<&(AdditionalBindingInfo, HashMap<&String, &BoundValue>)> =
            //     complete_bindings
            //         .par_iter()
            //         .filter(|(binding_info, binding)| {
            //             let c_next_node = nodes[1].clone();
            //             let next_evs = events_of_type.get(&c_next_node.event_type).unwrap();
            //             next_evs.iter().any(|next_event| {
            //                 c_next_node.clone().parents.iter().all(|p| {
            //                     match_dependency(
            //                         &p.dependency,
            //                         binding,
            //                         binding_info,
            //                         &next_event.id,
            //                         &event_map,
            //                         &object_map,
            //                     )
            //                 })
            //             })
            //         })
            //         .collect();
            sat_bindings.iter().for_each(|(info, binding)| {
                println!("Hello {:?}", info);
                let already_bound_vars: HashSet<&String> = binding.keys().map(|k| *k).collect();
                let new_bound_vars = get_new_bound_variables_from_node(
                    &next_node,
                    &events_of_type,
                    &already_bound_vars,
                );
                println!("#new_bound_vars: {}", new_bound_vars.len());
                let binding = combine_bindings(Some((info,binding)), &new_bound_vars);
                println!("#Combined bindings: {}", binding.len());
                todo!("CONTINUE");
                let sat_bindings = get_sat_bindings(
                    &next_next_node,
                    &binding,
                    &events_of_type,
                    &event_map,
                    &object_map,
                );
                println!(
                    "#Satisfied bindings: {} ({:.2}%)",
                    sat_bindings.len(),
                    100.0 * sat_bindings.len() as f32 / binding.len() as f32
                );
                println!(
                    "#Unsatisfied bindings: {} ({:.2}%)",
                    binding.len() - sat_bindings.len(),
                    100.0 * (binding.len() - sat_bindings.len()) as f32 / binding.len() as f32
                );
                if sat_bindings.len() > 0 {
                    println!("First sat binding: {:?}", sat_bindings.first().unwrap());
                }
            })
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    return Vec::new();
}

fn match_dependency(
    dep: &NodeDependency,
    variable_binding: &HashMap<&String, &BoundValue>,
    binding_info: &AdditionalBindingInfo,
    next_event_id: &String,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> bool {
    let next_ev = event_map.get(next_event_id).unwrap();
    let prev_ev = binding_info.past_events.last().unwrap();
    // Assuming eventually follows (for now)
    if next_ev.time < event_map.get(prev_ev).unwrap().time {
        return false;
    } else {
        let x = variable_binding.get(&dep.variable_name).unwrap();
        match dep.dependency_type {
            DependencyType::Simple => match x {
                BoundValue::Single(v) => match &next_ev.relationships {
                    Some(rels) => rels
                        .iter()
                        .any(|rel| rel.object_id == *v && rel.qualifier == dep.target_qualifier),
                    None => false,
                },
                BoundValue::Multiple(vs) => {
                    eprintln!(
                        "Expected single bound object for variable {}\nGot: {:?}",
                        dep.variable_name, vs
                    );
                    return false;
                }
            },
            DependencyType::All => match x {
                BoundValue::Single(v) => {
                    eprintln!(
                        "Expected multiple bound objects for variable {}\nGot: {:?}",
                        dep.variable_name, v
                    );
                    return false;
                }
                BoundValue::Multiple(vs) => match &next_ev.relationships {
                    Some(rels) => vs.iter().all(|v| {
                        rels.iter()
                            .any(|rel| rel.object_id == *v && rel.qualifier == dep.target_qualifier)
                    }),
                    None => false,
                },
            },
            DependencyType::ExistsInTarget => match x {
                BoundValue::Single(v) => match &next_ev.relationships {
                    Some(rels) => rels
                        .iter()
                        .any(|rel| rel.object_id == *v && rel.qualifier == dep.target_qualifier),
                    None => false,
                },
                BoundValue::Multiple(vs) => {
                    eprintln!(
                        "Expected single bound object for variable {}\nGot: {:?}",
                        dep.variable_name, vs
                    );
                    return false;
                }
            },
            DependencyType::ExistsInSource => match x {
                BoundValue::Single(v) => {
                    eprintln!(
                        "Expected multiple bound objects for variable {}\nGot: {:?}",
                        dep.variable_name, v
                    );
                    return false;
                }
                BoundValue::Multiple(vs) => match &next_ev.relationships {
                    Some(rels) => vs.iter().any(|v| {
                        rels.iter()
                            .any(|rel| rel.object_id == *v && rel.qualifier == dep.target_qualifier)
                    }),
                    None => false,
                },
            },
        }
    }
}

pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(nodes): Json<Vec<TreeNode>>,
) -> (StatusCode, Json<Option<Vec<String>>>) {
    with_ocel_from_state(&state, |ocel| {
        return (StatusCode::OK, Json(Some(check_with_tree(nodes, ocel))));
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}
