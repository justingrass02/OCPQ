use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
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
enum BoundValue {
    Single(String),
    Multiple(Vec<String>),
}

pub fn check_with_tree(nodes: Vec<TreeNode>, ocel: &OCEL) -> Vec<String> {
    let now = Instant::now();
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
    println!(
        "{} {} {} {}",
        event_map.len(),
        object_map.len(),
        events_of_type.len(),
        objects_of_type.len()
    );
    println!("Linking OCEL info took {:?}", now.elapsed());

    if nodes.len() > 0 {
        let current_node = nodes[0].clone();
        let evs = events_of_type.get(&current_node.event_type).unwrap();
        if current_node.children.len() > 0 {
            let required_bindings: HashSet<(String, String, DependencyType)> = current_node
                .children
                .into_iter()
                .map(|c| {
                    (
                        c.dependency.source_qualifier,
                        c.dependency.variable_name,
                        c.dependency.dependency_type,
                    )
                })
                .collect();
            let evs_bindings: Vec<HashMap<String, Vec<BoundValue>>> = evs
                .par_iter()
                .filter_map(|ev| match &ev.relationships {
                    Some(rels) => {
                        let mut binding_values: HashMap<String, Vec<BoundValue>> =
                            required_bindings
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
                        return Some(binding_values);
                    }
                    None => {
                        eprintln!("Child dependency but no linked objects");
                        return None;
                    }
                })
                .collect();

            println!("Required bindings: {}", evs_bindings.len());

            // Vec of tuple:
            //  event ids of previous events (Vec<String>)
            //  HashMap of variable -> value (i.e., object) assignments
            let complete_bindings: Vec<(Vec<String>, HashMap<&String, &BoundValue>)> = evs_bindings
                .par_iter()
                .enumerate()
                .flat_map(|(i, b)| {
                    if b.len() > 0 {
                        b.into_iter()
                            .filter(|(_, b)| b.len() > 0)
                            .map(|(a, bs)| bs.into_iter().map(move |b| (a, b)))
                            .multi_cartesian_product()
                            .map(|x| (vec![evs[i].id.clone()], x.into_iter().collect()))
                            .collect()
                    } else {
                        Vec::new()
                    }
                })
                .collect();
            println!("Total Combinations: {}", complete_bindings.len());
            println!("First combination: {:#?}", complete_bindings[0]);

            if nodes.len() >= 2 {
                let next_node = nodes[1].clone();
                let required_vars = next_node
                    .parents
                    .into_iter()
                    .map(|p| p.dependency.variable_name)
                    .collect_vec();
                println!("Required vars for second node: {:#?}", required_vars);

                let sat_bindings: Vec<&(Vec<String>, HashMap<&String, &BoundValue>)> =
                    complete_bindings
                        .par_iter()
                        .filter(|(prev_events, binding)| {
                            let c_next_node = nodes[1].clone();
                            let next_evs = events_of_type.get(&c_next_node.event_type).unwrap();
                            next_evs.iter().any(|next_event| {
                                c_next_node.clone().parents.iter().all(|p| {
                                    match_dependency(
                                        &p.dependency,
                                        binding,
                                        prev_events,
                                        &next_event.id,
                                        &event_map,
                                        &object_map,
                                    )
                                })
                            })
                        })
                        .collect();
                println!("Computed sat bindings. Total: {}", sat_bindings.len());
                println!(
                    "#Unsatisfied bindings: {}",
                    complete_bindings.len() - sat_bindings.len()
                );
                if sat_bindings.len() > 0 {
                    println!("First sat binding: {:?}", sat_bindings.first().unwrap());
                }
            }
        } else {
            println!("Finished with check (nothing to do)");
        }
    }
    return Vec::new();
}

fn match_dependency(
    dep: &NodeDependency,
    variable_binding: &HashMap<&String, &BoundValue>,
    prev_event_ids: &Vec<String>,
    next_event_id: &String,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> bool {
    let next_ev = event_map.get(next_event_id).unwrap();
    // Assuming eventually follows (for now)
    if next_ev.time < event_map.get(prev_event_ids.first().unwrap()).unwrap().time {
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
