use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AdditionalBindingInfo {
    past_events: Vec<String>,
}

fn match_and_add_new_bindings<'a>(
    prev_bindings_opt: Option<Bindings<'a>>,
    node: &'a TreeNode,
    events_of_type: &'a HashMap<String, Vec<&'a OCELEvent>>,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> Bindings<'a> {
    // if node.children.is_empty() && prev_bindings_opt.is_some() {
    //     // No variables to add
    //     return prev_bindings_opt.unwrap();
    // }
    // Get all events of the corresponding event type (determined by node)
    let events = events_of_type.get(&node.event_type).unwrap();
    let is_initial_binding = prev_bindings_opt.is_none();
    let mut prev_bindings: Bindings<'a> = match prev_bindings_opt {
        Some(p) => p,
        None => events
            .iter()
            .map(|ev| {
                (
                    AdditionalBindingInfo {
                        past_events: vec![ev.id.clone()],
                    },
                    HashMap::new(),
                )
            })
            .collect(),
    };
    println!(
        "is_initial_binding: {}; prev_bindings.len: {}",
        is_initial_binding,
        prev_bindings.len()
    );
    return prev_bindings
        .par_iter_mut()
        .filter_map(|(info, binding)| {
            let matching_event_opt = events
                .into_iter()
                .find(|ev| {
                    if is_initial_binding {
                        return ev.id == info.past_events[0];
                    }
                    for p in &node.parents {
                        match binding.get(&p.dependency.variable_name) {
                            Some(bound_val) => {
                                // TODO: Implement matching conditions here
                                // If no match: return false (early return)

                                let prev_ev = event_map.get(info.past_events.last().unwrap()).unwrap();
                                if prev_ev.time > ev.time {
                                    return false;
                                }
                                let ev_rels = ev.relationships.clone().unwrap();
                                let bound_val_match: bool  = match bound_val {
                                    BoundValue::Single(v) => {
                                        ev_rels.iter().any(|r| r.object_id == *v && r.qualifier == p.dependency.target_qualifier)
                                    },
                                    BoundValue::Multiple(vs) => {
                                        match p.dependency.dependency_type {
                                            DependencyType::All => {
                                                vs.iter().all(|v| {
                                                    ev_rels.iter().any(|r| r.object_id == *v && r.qualifier == p.dependency.target_qualifier)
                                                })
                                            },
                                            DependencyType::ExistsInTarget => {
                                                vs.iter().any(|v| {
                                                    ev_rels.iter().any(|r|
                                                        r.object_id == *v && r.qualifier == p.dependency.target_qualifier)
                                                })
                                            },
                                            DependencyType::ExistsInSource =>  todo!("ExistsInTarget type but multiple bound values;Should not happen"),
                                            DependencyType::Simple => todo!("Single type but multiple bound values;Should not happen"),
                                        }
                                    },
                                };
                                if !bound_val_match {
                                    return false;
                                }
                            }
                            None => panic!(
                                "Binding required variable which was not bound yet. ({})",
                                p.dependency.variable_name
                            ),
                        }
                    }
                    return true;
                });
            match matching_event_opt {
                Some(matching_event) => {
                    // We now got a matching event!
                    // We can now update the corresponding info...
                    if !is_initial_binding {
                        info.past_events.push(matching_event.id.clone());
                    }
                    // ...and also compute the updated binding
                    for c in &node.children {
                        // If binding already contains variable, there is nothing left to do :)
                        if !binding.contains_key(&c.dependency.variable_name) {
                            // Else... construct bound value and insert it
                            match matching_event.relationships.clone() {
                                Some(rel) => {
                                    let new_bound_value: BoundValue =
                                        match c.dependency.dependency_type {
                                            DependencyType::Simple => {
                                                let obj_id: String = rel
                                                    .into_iter()
                                                    .find(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .unwrap()
                                                    .object_id;
                                                BoundValue::Single(obj_id)
                                            }
                                            DependencyType::All => {
                                                let obj_ids: Vec<String> = rel
                                                    .into_iter()
                                                    .filter(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .map(|r| r.object_id)
                                                    .collect();
                                                BoundValue::Multiple(obj_ids)
                                            }
                                            DependencyType::ExistsInSource => {
                                                let obj_ids: Vec<String> = rel
                                                    .into_iter()
                                                    .filter(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .map(|r| r.object_id)
                                                    .collect();
                                                BoundValue::Multiple(obj_ids)
                                            }
                                            DependencyType::ExistsInTarget => {
                                                let obj_id: String = rel
                                                    .into_iter()
                                                    .find(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .unwrap()
                                                    .object_id;
                                                BoundValue::Single(obj_id)
                                            }
                                        };
                                    binding.insert(
                                        c.dependency.variable_name.clone(),
                                        new_bound_value,
                                    );
                                }
                                None => {
                                    panic!(
                                        "Expected relationship in event with id {} (of type {})",
                                        matching_event.id, node.event_type
                                    );
                                }
                            }
                        }
                    }

                    return Some((info.to_owned(), binding.to_owned()));
                }
                None => {
                    // No matching event found.
                    // println!("Found no matching event for binding: {:?} with info {:?}",binding, info);
                    return None;
                }
            }
        })
        .collect();
}

fn combine_bindings<'a>(
    prev_binding: Option<(AdditionalBindingInfo, HashMap<String, BoundValue>)>,
    new_vars_with_value: Vec<(AdditionalBindingInfo, HashMap<String, Vec<BoundValue>>)>,
) -> Vec<(AdditionalBindingInfo, HashMap<String, BoundValue>)> {
    let new_bindings: Vec<(AdditionalBindingInfo, HashMap<String, BoundValue>)> =
        new_vars_with_value
            .into_par_iter()
            .flat_map(|(info, b)| {
                if b.len() > 0 {
                    let cat = b
                        .into_iter()
                        .filter(|(_, b)| b.len() > 0)
                        .map(|(a, bs)| bs.into_iter().map(move |b| (a.clone(), b)).collect_vec())
                        .multi_cartesian_product();
                    let xxx: Vec<(AdditionalBindingInfo, HashMap<String, BoundValue>)> =
                        match prev_binding.clone() {
                            Some((prev_info, prev_b)) => cat
                                .cartesian_product(prev_b.iter())
                                .flat_map(|(new, (x, y))| vec![new, vec![(x.clone(), y.clone())]])
                                .map(|x| (info.clone(), x.into_iter().collect()))
                                .collect(),
                            None => cat
                                .map(|x| (info.clone(), x.into_iter().collect()))
                                .collect(),
                        };
                    return xxx;
                } else {
                    match prev_binding.clone() {
                        Some((prev_info, prev_b)) => {
                            let xxx: Vec<(AdditionalBindingInfo, HashMap<String, BoundValue>)> =
                                vec![(prev_info, prev_b)];
                            return xxx;
                        }
                        None => Vec::new(),
                    }
                }
            })
            .collect();

    return new_bindings;
}
type Bindings<'a> = Vec<(AdditionalBindingInfo, HashMap<String, BoundValue>)>;

pub fn get_sat_bindings<'a>(
    node: &TreeNode,
    binding: &'a Bindings<'a>,
    events_of_type: &HashMap<String, Vec<&OCELEvent>>,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> Vec<&'a (AdditionalBindingInfo, HashMap<String, BoundValue>)> {
    let sat_bindings: Vec<&(AdditionalBindingInfo, HashMap<String, BoundValue>)> = binding
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
        let mut bindings: Option<Bindings> = None;
        for i in 0..nodes.len() {
            let node = &nodes[i];
            bindings = Some(match_and_add_new_bindings(
                bindings,
                node,
                &events_of_type,
                &event_map,
                &object_map,
            ));
            println!(
                "#Bindings for i={}: {}",
                i,
                bindings.as_ref().unwrap().len()
            )
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    return Vec::new();
}

fn match_dependency(
    dep: &NodeDependency,
    variable_binding: &HashMap<String, BoundValue>,
    binding_info: &AdditionalBindingInfo,
    next_event_id: &String,
    event_map: &HashMap<String, &OCELEvent>,
    object_map: &HashMap<String, &OCELObject>,
) -> bool {
    let next_ev = event_map.get(next_event_id).unwrap();
    let prev_ev = event_map
        .get(binding_info.past_events.last().unwrap())
        .unwrap();
    // Assuming eventually follows (for now)
    if next_ev.time < prev_ev.time {
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
