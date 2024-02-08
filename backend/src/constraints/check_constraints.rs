use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use axum::{extract::State, http::StatusCode, Json};
use itertools::Itertools;
use process_mining::event_log::ocel::ocel_struct::{OCELEvent, OCEL};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constraints::ObjectVariable,
    preprocessing::preprocess::{
        get_events_of_type_associated_with_objects, link_ocel_info, LinkedOCEL,
    },
    with_ocel_from_state, AppState,
};

use super::{AdditionalBindingInfo, BoundValue, PastEventInfo, TreeNode};

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
                                BoundValue::Single(v) => &r.object_id == v,
                                BoundValue::Multiple(_) => todo!(),
                            },
                            None => false,
                        }
                    });
                }

                None => {
                    return false;
                }
            }
        }
    }
    true
}

fn match_and_add_new_bindings<'a>(
    prev_bindings_opt: Option<Bindings>,
    node: &'a TreeNode,
    linked_ocel: &'a LinkedOCEL,
) -> Vec<(Binding, Option<ViolationReason>)> {
    // Get all events of the corresponding event type (determined by node)
    // let events = linked_ocel.events_of_type.get(&node.event_type).unwrap();
    // let is_initial_binding = prev_bindings_opt.is_none();
    let mut prev_bindings: Bindings = match prev_bindings_opt {
        Some(p) => p,
        None => {
            get_events_of_type_associated_with_objects(linked_ocel, &node.event_type, Vec::new())
                .iter()
                .map(|ev| {
                    (
                        AdditionalBindingInfo {
                            past_events: vec![PastEventInfo {
                                event_id: ev.id.clone(),
                                node_id: node.id.clone(),
                            }],
                        },
                        HashMap::new(),
                    )
                })
                .collect()
        }
    };
    // Iterate over all bindings, updating them
    return prev_bindings
        .par_iter_mut()
        .flat_map(|(info, binding)| {
            // Get all matching events (i.e. events with correct association to unbound objects)
            let matching_events = get_events_of_type_associated_with_objects(
                linked_ocel,
                &node.event_type,
                get_object_ids_from_node_and_binding(node, binding),
            )
            .into_par_iter()
            .filter(|ev| {
                // First check for correct related objects
                if !event_has_correct_objects(node, ev, binding) {
                    return false;
                }

                // Then check time difference
                for p in &node.parents {
                    let last_prev_ev = info
                        .past_events
                        .iter()
                        .filter(|ev| ev.node_id == p.id)
                        .last();
                    match last_prev_ev {
                        Some(prev_ev) => {
                            let second_diff = (ev.time
                                - linked_ocel.event_map.get(&prev_ev.event_id).unwrap().time)
                                .num_seconds();
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
                true
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
                _ => num_matching_events,
            };

            matching_events
                .take_any(take)
                .flat_map(|matching_event| {
                    let mut info_cc: AdditionalBindingInfo = info.clone();
                    let binding: HashMap<String, BoundValue> = binding.clone();
                    // We now got one or more matching event(s)!
                    // We can now update the corresponding info...
                    // if !is_initial_binding {
                    info_cc.past_events.push(PastEventInfo {
                        event_id: matching_event.id.clone(),
                        node_id: node.id.clone(),
                    });
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
                                        .iter()
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
                                                    cloned_bind
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
                    bindings
                })
                .collect::<Vec<(Binding, Option<ViolationReason>)>>()
        })
        .collect();
}

fn get_object_ids_from_node_and_binding(
    node: &TreeNode,
    binding: &HashMap<String, BoundValue>,
) -> Vec<String> {
    return node
        .variables
        .iter()
        .filter(|v| !v.bound)
        .filter_map(|v| match binding.get(&v.variable.name) {
            Some(BoundValue::Single(s)) => Some(s.clone()),
            Some(BoundValue::Multiple(_)) => {
                eprintln!("Multiple objects?");
                None
            }
            None => {
                eprintln!("Variable unbound?");
                None
            }
        })
        .collect_vec();
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

fn check_with_tree(
    variables: Vec<ObjectVariable>,
    nodes: Vec<TreeNode>,
    ocel: &OCEL,
) -> (Vec<usize>, Vec<Violations>) {
    let linked_ocel = link_ocel_info(ocel);
    let mut binding_sizes: Vec<usize> = Vec::new();
    let mut violation_sizes: Vec<usize> = Vec::new();
    let mut violations: Vec<Violations> = Vec::new();
    println!("Variables: {:?}", variables);
    let now = Instant::now();
    if !nodes.is_empty() {
        let mut bindings: Bindings = vec![(
            AdditionalBindingInfo {
                past_events: Vec::new(),
            },
            HashMap::new(),
        )];
        let initially_bound_vars = variables.iter().filter(|v| v.initially_bound).collect_vec();
        for v in initially_bound_vars {
            bindings = bindings
                .into_iter()
                .flat_map(|(add_info, bound_val)| match &v.o2o {
                    Some(o2o) => {
                        // println!(bound_val)
                        let bound_parent_object_val =
                            bound_val.get(&o2o.parent_variable_name).unwrap();
                        match bound_parent_object_val {
                            BoundValue::Single(parent_object_id) => {
                                let parent_object =
                                    linked_ocel.object_map.get(parent_object_id).unwrap();
                                match &parent_object.relationships {
                                    Some(rels) => rels
                                        .iter()
                                        .filter(|r| r.qualifier == o2o.qualifier)
                                        .map(|r| {
                                            let mut new_bound_val = bound_val.clone();
                                            new_bound_val.insert(
                                                v.name.clone(),
                                                BoundValue::Single(r.object_id.clone()),
                                            );
                                            (add_info.clone(), new_bound_val)
                                        })
                                        .collect_vec(),
                                    None => todo!("No rels in parent {}", parent_object.id),
                                }
                            }
                            BoundValue::Multiple(_) => todo!(),
                        }
                    }
                    None => linked_ocel
                        .objects_of_type
                        .get(&v.object_type)
                        .unwrap()
                        .iter()
                        .map(|obj| {
                            let mut new_bound_val = bound_val.clone();
                            new_bound_val
                                .insert(v.name.clone(), BoundValue::Single(obj.id.clone()));
                            (add_info.clone(), new_bound_val)
                        })
                        .collect_vec(),
                })
                .collect();
        }

        println!("#Bindings (initial): {}", bindings.len());

        for node in &nodes {
            // let node = &nodes[i];
            if node.children.is_empty() && node.parents.is_empty() {
                // Here we just check the count constraints & do not update bindings
                // Instead, we only gather first violations
                if node.count_constraint.min > 0 || node.count_constraint.max < usize::MAX {
                    println!("Node {} has no child/parent", { &node.id });
                    let x = match_and_add_new_bindings(Some(bindings.clone()), node, &linked_ocel);
                    violations.push(
                        x.into_iter()
                            .filter_map(|(b, violation)| violation.map(|v| ((b), v)))
                            .collect_vec(),
                    );
                } else {
                    violations.push(Vec::new());
                }
                continue;
            }
            let x = match_and_add_new_bindings(Some(bindings), node, &linked_ocel);
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
                "#Bindings for node {}: {};\nViolations: {}",
                node.id,
                bindings.len(),
                new_violations.len(),
            );
            violations.push(new_violations);
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    println!("No connected node left!");
    println!("Finished in {:?}", now.elapsed());
    (binding_sizes, violations)
}

#[derive(Serialize, Deserialize)]
pub struct CheckWithTreeRequest {
    pub variables: Vec<ObjectVariable>,
    #[serde(rename = "nodesOrder")]
    pub nodes_order: Vec<TreeNode>,
}
pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(req): Json<CheckWithTreeRequest>,
) -> (StatusCode, Json<Option<(Vec<usize>, Vec<Violations>)>>) {
    with_ocel_from_state(&state, |ocel| {
        (
            StatusCode::OK,
            Json(Some(check_with_tree(req.variables, req.nodes_order, ocel))),
        )
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}
