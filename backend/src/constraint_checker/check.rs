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
struct TimeConstraint {
    #[serde(rename = "minSeconds")]
    min_seconds: f64,
    #[serde(rename = "maxSeconds")]
    max_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Connection {
    #[serde(rename = "type")]
    connection_type: ConstraintType,
    #[serde(rename = "timeConstraint")]
    time_constraint: TimeConstraint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectVariable {
    name: String,
    #[serde(rename = "type")]
    object_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectedVariable {
    variable: ObjectVariable,
    qualifier: String,
    bound: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CountConstraint {
    min: usize,
    max: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNode {
    #[serde(rename = "eventType")]
    event_type: String,
    parents: Vec<TreeNodeConnection>,
    children: Vec<TreeNodeConnection>,
    variables: Vec<SelectedVariable>,
    #[serde(rename = "countConstraint")]
    count_constraint: CountConstraint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TreeNodeConnection {
    connection: Connection,
    #[serde(rename = "eventType")]
    event_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        .object_types
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdditionalBindingInfo {
    past_events: Vec<String>,
}

///
/// Check for all (unbound) variables of node if given event is associated with the correct qualified objects
///
fn event_has_correct_objects(
    node: &TreeNode,
    ev: &OCELEvent,
    binding: &HashMap<String, BoundValue>,
) -> bool {
    for v in &node.variables {
        if !v.bound {
            match &ev.relationships {
                Some(rel) => {
                    // Find relationship with corresponding qualifier
                    return rel.iter().any(|r| {
                        if r.qualifier != v.qualifier {
                            return false;
                        }
                        match binding.get(&v.variable.name) {
                            Some(bound_val) => match bound_val {
                                BoundValue::Single(v) => {
                                    if &r.object_id == v {
                                        return true;
                                    } else {
                                        return false;
                                    }
                                }
                                BoundValue::Multiple(_) => todo!(),
                            },
                            None => {
                                return false;
                            }
                        }
                    });
                }

                None => {
                    return false;
                }
            }
        }
    }
    return true;
}

fn match_and_add_new_bindings<'a>(
    prev_bindings_opt: Option<Bindings>,
    node: &'a TreeNode,
    events_of_type: &'a HashMap<String, Vec<&'a OCELEvent>>,
    event_map: &HashMap<String, &OCELEvent>,
    _object_map: &HashMap<String, &OCELObject>,
) -> Vec<(Binding, Option<ViolationReason>)> {
    // Get all events of the corresponding event type (determined by node)
    let events = events_of_type.get(&node.event_type).unwrap();
    // let is_initial_binding = prev_bindings_opt.is_none();
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
    // println!(
    //     "is_initial_binding: {}; prev_bindings.len: {}, parents: {:#?}\n\n children: {:#?}\n\n",
    //     is_initial_binding,
    //     prev_bindings.len(),
    //     node.parents,
    //     node.children
    // );

    // Iterate over all bindings, updating them
    return prev_bindings
        .par_iter_mut()
        .flat_map(|(info, binding)| {
            // Get all matching events (i.e. events with correct association to unbound objects)
            let matching_events = events.into_par_iter().filter(|ev| {
                // First check for correct related objects
                if !event_has_correct_objects(node, ev, &binding) {
                    return false;
                }

                // Then check time difference
                for p in &node.parents {
                    let last_ev_id = info
                        .past_events
                        .iter()
                        .filter(|ev| event_map.get(*ev).unwrap().event_type == p.event_type)
                        .last();
                    match last_ev_id {
                        Some(ev_id) => {
                            let second_diff =
                                (ev.time - event_map.get(ev_id).unwrap().time).num_seconds();
                            if (second_diff as f64) < p.connection.time_constraint.min_seconds
                                || (second_diff as f64) > p.connection.time_constraint.max_seconds
                            {
                                return false;
                            }
                        }
                        None => {
                            // Hmm weird... no matching parent found?!
                            eprintln!("No matching parent found for event type {}", p.event_type);
                        }
                    }
                }
                return true;
            });
            let num_matching_events = matching_events.clone().count();

            if num_matching_events < node.count_constraint.min {
                return vec![(
                    (info.clone(), binding.clone()),
                    Some(ViolationReason::TooFewMatchingEvents),
                )];
            } else if num_matching_events > node.count_constraint.max {
                return vec![(
                    (info.clone(), binding.clone()),
                    Some(ViolationReason::TooManyMatchingEvents),
                )];
            }

            let take = match node.parents.len() {
                0 => 1,
                _ => events.len(),
            };

            matching_events
                .take_any(take)
                .flat_map(|matching_event| {
                    let mut info_cc: AdditionalBindingInfo = info.clone();
                    let binding: HashMap<String, BoundValue> = binding.clone();
                    // We now got one or more matching event(s)!
                    // We can now update the corresponding info...
                    // if !is_initial_binding {
                    info_cc.past_events.push(matching_event.id.clone());
                    // }

                    let mut bindings: Vec<(Binding, Option<ViolationReason>)> =
                        vec![((info_cc, binding.clone()), None)];
                    for v in &node.variables {
                        if v.bound {
                            // v should be binded!
                            // First gather possible values
                            match &matching_event.relationships {
                                Some(rels) => {
                                    let matching_qualified_relationships = rels
                                        .into_iter()
                                        .filter(|rel| rel.qualifier == v.qualifier)
                                        .collect_vec();
                                    bindings = bindings
                                        .into_iter()
                                        .flat_map(|binding| {
                                            matching_qualified_relationships
                                                .iter()
                                                .map(|matching_rel| {
                                                    let mut cloned_bind = binding.clone();
                                                    cloned_bind.0 .1.insert(
                                                        v.variable.name.clone(),
                                                        BoundValue::Single(
                                                            matching_rel.object_id.clone(),
                                                        ),
                                                    );
                                                    return cloned_bind;
                                                })
                                                .collect_vec()
                                        })
                                        .collect_vec();
                                }
                                None => {
                                    // New variable cannot be bound as matching_event has no object relationships
                                }
                            }
                        }
                    }
                    // }
                    return bindings;
                })
                .collect::<Vec<(Binding, Option<ViolationReason>)>>()
        })
        .collect();
}

type Binding = (AdditionalBindingInfo, HashMap<String, BoundValue>);
type Bindings = Vec<Binding>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ViolationReason {
    TooFewMatchingEvents,
    TooManyMatchingEvents,
    // NoMatchingEvents,
    // MultipleMatchingEvents,
    // MatchingEvents,
}
type Violations = Vec<(Binding, ViolationReason)>;

pub fn check_with_tree(nodes: Vec<TreeNode>, ocel: &OCEL) -> (Vec<usize>, Vec<Violations>) {
    let LinkedOCEL {
        event_map,
        object_map,
        events_of_type,
        objects_of_type,
    } = link_ocel_info(ocel);
    let mut binding_sizes: Vec<usize> = Vec::new();
    let mut violation_sizes: Vec<usize> = Vec::new();
    let mut violations : Vec<Violations> = Vec::new();
    if nodes.len() > 0 {
        let mut bindings: Bindings = vec![(
            AdditionalBindingInfo {
                past_events: Vec::new(),
            },
            HashMap::new(),
        )];

        // TODO: For now just start with unbound variables of first node
        // This should be updated later (bc. we might have some unbound variables at a later node)

        let first_node = nodes.get(0).unwrap();
        for v in &first_node.variables {
            if !v.bound {
                bindings = bindings
                    .into_iter()
                    .flat_map(|(add_info, bound_val)| {
                        objects_of_type
                            .get(&v.variable.object_type)
                            .unwrap()
                            .iter()
                            .map(|obj| {
                                let mut new_bound_val = bound_val.clone();
                                new_bound_val.insert(
                                    v.variable.name.clone(),
                                    BoundValue::Single(obj.id.clone()),
                                );
                                return (add_info.clone(), new_bound_val);
                            })
                            .collect_vec()
                        // vec![b]
                    })
                    .collect();
            }
        }

        println!("#Bindings (initial): {}", bindings.len());

        for i in 0..nodes.len() {
            let node = &nodes[i];
            if node.children.is_empty() && node.parents.is_empty() {
                println!("Node {} has no child/parent", { i });
                continue;
            }
            let x = Some(match_and_add_new_bindings(
                Some(bindings),
                node,
                &events_of_type,
                &event_map,
                &object_map,
            ))
            .unwrap();
            let mut new_bindings: Bindings = Vec::new();
            let mut new_violations: Violations = Vec::new();
            for (b, v) in x {
                match v {
                    Some(violation) => {
                        new_violations.push(((b), violation));
                    }
                    None => {
                        new_bindings.push(b);
                    }
                }
            }
            binding_sizes.push(new_bindings.len());
            violation_sizes.push(new_violations.len());
            bindings = new_bindings;
            println!(
                "#Bindings for i={}: {};\nViolations: {}\n{:?}",
                i,
                bindings.len(),
                new_violations.len(),
                if new_violations.len() > 0 {
                    Some(new_violations[0].1.clone())
                } else {
                    None
                }
            );
            violations.push(new_violations);
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    println!("No connected node left!");
    return (binding_sizes, violations);
}

pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(nodes): Json<Vec<TreeNode>>,
) -> (StatusCode, Json<Option<(Vec<usize>, Vec<Violations>)>>) {
    with_ocel_from_state(&state, |ocel| {
        return (StatusCode::OK, Json(Some(check_with_tree(nodes, ocel))));
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}
