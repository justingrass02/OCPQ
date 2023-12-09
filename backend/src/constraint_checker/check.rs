use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
enum ConstraintType {
    #[serde(rename = "response")]
    Response,
    #[serde(rename = "unary-response")]
    UnaryResponse,
    #[serde(rename = "non-response")]
    NonResponse,
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
    #[serde(rename = "constraintType")]
    constraint_type: ConstraintType,
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
    #[allow(dead_code)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AdditionalBindingInfo {
    past_events: Vec<String>,
}

fn match_and_add_new_bindings<'a>(
    prev_bindings_opt: Option<Bindings>,
    node: &'a TreeNode,
    events_of_type: &'a HashMap<String, Vec<&'a OCELEvent>>,
    event_map: &HashMap<String, &OCELEvent>,
    _object_map: &HashMap<String, &OCELObject>,
) -> Vec<(Binding,Option<ViolationReason>)> {
    // Get all events of the corresponding event type (determined by node)
    let events = events_of_type.get(&node.event_type).unwrap();
    let is_initial_binding = prev_bindings_opt.is_none();
    let mut prev_bindings: Bindings = match prev_bindings_opt {
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
        "is_initial_binding: {}; prev_bindings.len: {}, parents: {:#?}\n\n children: {:#?}\n\n",
        is_initial_binding,
        prev_bindings.len(),
        node.parents,
        node.children
    );
    // Iterate over all bindings, updating them
    return prev_bindings
        .par_iter_mut()
        .flat_map(|(info, binding)| {
            let matching_events = events
            .into_par_iter()
            .filter(|ev| {
                if is_initial_binding {
                        return &ev.id == &info.past_events[0];
                    }
                    for p in &node.parents {
                        match binding.get(&p.dependency.variable_name) {
                            Some(bound_val) => {
                                
                                let prev_ev = event_map.get(info.past_events.last().unwrap()).unwrap();
                                match p.dependency.constraint_type {
                                    ConstraintType::Response => {
                                        // If no match: return false (early return)
                                        if prev_ev.time > ev.time {
                                            return false;
                                        }
                                    },
                                    ConstraintType::UnaryResponse => {
                                        if prev_ev.time > ev.time {
                                            return false;
                                        }
                                    },
                                    ConstraintType::NonResponse => {
                                        if prev_ev.time > ev.time {
                                            return false;
                                        }
                                    },
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
                                            DependencyType::ExistsInSource => {
                                                vs.iter().any(|v| {
                                                    ev_rels.iter().any(|r|
                                                        r.object_id == *v && r.qualifier == p.dependency.target_qualifier)
                                                })
                                            },
                                            DependencyType::ExistsInTarget =>  todo!("ExistsInTarget type but multiple bound values {}", p.dependency.variable_name),
                                            DependencyType::Simple => todo!("Single type but multiple bound values;Should not happen {}", p.dependency.variable_name),
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

                let num_matching_events = matching_events.clone().count();
                // In the future, we should distinguish between (the *number* of) new bindings and whether the constraints are satisfied
                // So far, we simply return Vec::new() if some constraint (like NonResponse or Unary Response) is violated
                if node.parents.iter().any(|p| p.dependency.constraint_type == ConstraintType::NonResponse) {
                    if num_matching_events > 0 {
                        return vec![((info.clone(),binding.clone()),Some(ViolationReason::MatchingEvents))];
                        }else{
                            return vec![((info.clone(),binding.clone()),None)];
                        }
                    }else if node.parents.iter().any(|p| p.dependency.constraint_type == ConstraintType::UnaryResponse){
                        if num_matching_events > 1 {
                            return vec![((info.clone(),binding.clone()),Some(ViolationReason::MultipleMatchingEvents))];
                        }
                    }
            if num_matching_events == 0 {
                return vec![((info.clone(),binding.clone()),Some(ViolationReason::NoMatchingEvents))];
            }
                let take = match node.parents.len() {
                    0 => 1,
                    _ => events.len()
                };

                matching_events.take_any(take).flat_map(|matching_event| {
                    let mut info_cc: AdditionalBindingInfo = info.clone();
                    // let binding: HashMap<String, BoundValue> = binding.clone();
                    // We now got a matching event!
                    // We can now update the corresponding info...
                    if !is_initial_binding {
                        info_cc.past_events.push(matching_event.id.clone());
                    }

                    let mut bindings: Vec<(Binding,Option<ViolationReason>)> = vec![((info_cc,binding.clone()),None)];
                    // ...and also compute the updated bindings
                    for c in &node.children {
                        // If binding already contains variable, there is nothing left to do :)
                        if !binding.contains_key(&c.dependency.variable_name) {
                            // Else... construct bound value and insert it
                            match matching_event.relationships.clone() {
                                Some(rel) => {
                                    let new_bound_value_opt: Option<Vec<BoundValue>> =
                                        match c.dependency.dependency_type {
                                            DependencyType::Simple => {
                                                let obj_rel_opt = rel
                                                    .into_iter()
                                                    .find(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    });
                                                match obj_rel_opt {
                                                    Some(obj_rel) => {
                                                    Some(vec![BoundValue::Single(obj_rel.object_id)])
                                                    },
                                                    None => None,
                                                }
                                            }
                                            DependencyType::All => {
                                                let obj_ids: Vec<String> = rel
                                                    .into_iter()
                                                    .filter(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .map(|r| r.object_id)
                                                    .collect();
                                                Some(vec![BoundValue::Multiple(obj_ids)])
                                            }
                                            DependencyType::ExistsInSource => {
                                                let obj_ids: Vec<String> = rel
                                                    .into_iter()
                                                    .filter(|r| {
                                                        r.qualifier == c.dependency.source_qualifier
                                                    })
                                                    .map(|r| r.object_id)
                                                    .collect();
                                                Some(obj_ids.into_iter().map(|o_id| BoundValue::Single(o_id)).collect())
                                            }
                                            DependencyType::ExistsInTarget => {
                                                let obj_rel_opt = rel
                                                .into_iter()
                                                .find(|r| {
                                                    r.qualifier == c.dependency.source_qualifier
                                                });
                                            match obj_rel_opt {
                                                Some(obj_rel) => {
                                                Some(vec![BoundValue::Single(obj_rel.object_id)])
                                                },
                                                None => None,
                                            }
                                            }
                                        };
                                    match new_bound_value_opt {
                                        Some(new_bound_value) => {
                                           bindings = bindings.into_iter().flat_map(|binding| {
                                                new_bound_value.iter().map(|new_bound_value| {
                                                    let mut cloned_bind = binding.clone();
                                                    cloned_bind.0.1.insert(c.dependency.variable_name.clone(), new_bound_value.clone());
                                                    return cloned_bind;
                                                }).collect_vec()
                                            }).collect_vec();

                                        },
                                        None => {
                                            return Vec::new()
                                        },
                                    }
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
                    return bindings;
                }).collect::<Vec<(Binding,Option<ViolationReason>)>>()
        })
        .collect();
}


type Binding = (AdditionalBindingInfo, HashMap<String, BoundValue>);
type Bindings = Vec<Binding>;
#[derive(Clone, Debug)]
enum ViolationReason {
    NoMatchingEvents,
    MultipleMatchingEvents,
    MatchingEvents,
}
type Violations = Vec<(Binding,ViolationReason)>;

pub fn check_with_tree(nodes: Vec<TreeNode>, ocel: &OCEL) -> (Vec<usize>,Vec<usize>) {
    let LinkedOCEL {
        event_map,
        object_map,
        events_of_type,
        objects_of_type: _,
    } = link_ocel_info(ocel);
    let mut binding_sizes: Vec<usize> = Vec::new();
    let mut violation_sizes: Vec<usize> = Vec::new();
    if nodes.len() > 0 {
        // let mut bindings: Option<Vec<(Binding,Option<ViolationReason>)>> = None;
        let mut bindings: Option<Bindings> = None;
        for i in 0..nodes.len() {
            let node = &nodes[i];
            if node.children.is_empty() && node.parents.is_empty() {
                println!("Node {} has no child/parent", { i });
                continue;
            }
            let x = Some(match_and_add_new_bindings(
                bindings.clone(),
                node,
                &events_of_type,
                &event_map,
                &object_map,
            )).unwrap();
            let mut new_bindings: Bindings = Vec::new();
            let mut new_violations: Violations = Vec::new();
            for (b,v) in x {
                match v {
                    Some(violation) => {
                        new_violations.push(((b),violation));
                    },
                    None => {
                        new_bindings.push(b);
                    },
                }
            }
            binding_sizes.push(new_bindings.len());
            violation_sizes.push(new_violations.len());
            bindings = Some(new_bindings);
            println!(
                "#Bindings for i={}: {};\nViolations: {}\n{:?}",
                i,
                bindings.as_ref().unwrap().len(),
                new_violations.len(),
                if new_violations.len() > 0 { Some(new_violations[0].1.clone()) } else { None}
            )
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    println!("No connected node left!");
    return (binding_sizes,violation_sizes);
}

pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(nodes): Json<Vec<TreeNode>>,
) -> (StatusCode, Json<Option<(Vec<usize>,Vec<usize>)>>) {
    with_ocel_from_state(&state, |ocel| {
        return (StatusCode::OK, Json(Some(check_with_tree(nodes, ocel))));
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}
