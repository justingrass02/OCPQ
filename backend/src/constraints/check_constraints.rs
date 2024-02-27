use std::{collections::HashMap, time::Instant};

use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use process_mining::event_log::ocel::ocel_struct::{OCELEvent, OCEL};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constraints::{FirstOrLastEventOfType, ObjectVariable},
    preprocessing::preprocess::{
        get_events_of_type_associated_with_objects, link_ocel_info, LinkedOCEL,
    },
    with_ocel_from_state, AppState,
};

use super::{
    AdditionalBindingInfo, BoundValue, EventTreeNode, PastEventInfo, TreeNode, TreeNodeType,
};

///
/// Check for all (unbound) variables of node if given event is associated with the correct qualified objects
///
fn event_has_correct_objects(
    node: &EventTreeNode,
    ev: &OCELEvent,
    binding: &HashMap<String, BoundValue>,
) -> bool {
    for v in &node.variables {
        if !v.bound {
            match &ev.relationships {
                Some(rel) => {
                    // Find relationship with corresponding qualifier
                    return rel.iter().any(|r| {
                        match &v.qualifier {
                            Some(q) => {
                                if &r.qualifier != q {
                                    return false;
                                }
                            }
                            // If qualifier is None, then we do not want to filter based on qualifier
                            None => {}
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
    node: &'a EventTreeNode,
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
    prev_bindings
        .par_iter_mut()
        .flat_map(|(info, binding)| {
            // Get all matching events (i.e. events with correct association to unbound objects)
            let mut matching_events: Vec<&OCELEvent> = get_events_of_type_associated_with_objects(
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
                true
            })
            .collect();
        let num_matching_events = matching_events.clone().len();
        let mut take = num_matching_events;
        let mut skip = 0;
        match node.first_or_last_event_of_type {
            Some(FirstOrLastEventOfType::First) => {
                take = 1.min(num_matching_events);
            }
            Some(FirstOrLastEventOfType::Last) => {
                take = 1.min(num_matching_events);
                skip = num_matching_events - 1;
            }
            None => {}
        }
        matching_events = matching_events.into_iter()
        .sorted_by_key(|ev| ev.time).skip(skip).take(take).filter(|ev| {
            if !event_satisfies_num_qualifiers_constraint(node, ev) {
                return false;
            }
            if !event_satisfies_waiting_time_constraint(node, ev, linked_ocel) {
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
                        eprintln!("No matching parent found for event type {:?}; Past events: {:?}", p.event_type, info.past_events);
                    }
                }
            }
            true
        }).collect();
        let num_matching_events_after = matching_events.len();
            if num_matching_events_after < node.count_constraint.min {
                return vec![(
                    (info.clone(), binding.clone()),
                    Some(ViolationReason::TooFewMatchingEvents),
                )];
            } else if num_matching_events_after > node.count_constraint.max {
                return vec![(
                    (info.clone(), binding.clone()),
                    Some(ViolationReason::TooManyMatchingEvents),
                )];
            }
            matching_events
                .iter()
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
                            // v should be bound!
                            // First gather possible values
                            match &matching_event.relationships {
                                Some(rels) => {
                                    let matching_qualified_relationships = rels
                                        .iter()
                                        .filter(|rel| match &v.qualifier {
                                            Some(q) => &rel.qualifier == q,
                                            None => true,
                                        })
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
                                    eprintln!("Variable {} should be bound but event does not have object relationships?", v.variable.name);
                                }
                            }
                        }
                    }
                    // }
                    bindings
                })
                .collect::<Vec<(Binding, Option<ViolationReason>)>>()
        })
        .collect()
}
fn event_satisfies_waiting_time_constraint(
    node: &EventTreeNode,
    ev: &&OCELEvent,
    linked_ocel: &LinkedOCEL<'_>,
) -> bool {
    match &node.waiting_time_constraint {
        Some(waiting_time_range) => {
            let object_ids: Vec<String> = match &ev.relationships {
                Some(rels) => rels.iter().map(|r| r.object_id.clone()).collect(),
                None => Vec::new(),
            };
            let mut last_prev_event_time: Option<DateTime<Utc>> = None;
            for object_id in &object_ids {
                match linked_ocel.object_events_map.get(object_id) {
                    None => {
                        eprint!("Object ID {} has not entry in object_events_map", object_id)
                    }
                    Some(object_ids) => {
                        for e_id in object_ids {
                            match linked_ocel.event_map.get(e_id) {
                                Some(e) => {
                                    // Event has to happen before target event (to be considered for waiting time)
                                    if e.time < ev.time
                                        && (last_prev_event_time.is_none()
                                            || e.time > last_prev_event_time.unwrap())
                                    {
                                        last_prev_event_time = Some(e.time);
                                    }
                                }
                                None => eprintln!("Event {} has no entry in event_map", e_id),
                            }
                        }
                    }
                }
            }
            let difference_seconds = match last_prev_event_time {
                Some(prev_time) => (ev.time - prev_time).num_seconds(),
                None => 0,
            };
            (difference_seconds as f64) >= waiting_time_range.min_seconds
                && (difference_seconds as f64) <= waiting_time_range.max_seconds
        }
        None => true,
    }
}

fn event_satisfies_num_qualifiers_constraint(node: &EventTreeNode, ev: &&OCELEvent) -> bool {
    match &node.num_qualified_objects_constraint {
        Some(num_qual_constr) => {
            let mut ev_qualifier_nums: HashMap<&String, usize> = HashMap::new();
            match &ev.relationships {
                Some(rels) => {
                    for r in rels {
                        *ev_qualifier_nums.entry(&r.qualifier).or_insert(0) += 1;
                    }
                }
                None => {}
            }
            num_qual_constr.iter().all(|(qual, num_constr)| {
                let n = *ev_qualifier_nums.get(qual).unwrap_or(&0);
                n <= num_constr.max && n >= num_constr.min
            })
        }
        None => true,
    }
}

fn get_object_ids_from_node_and_binding(
    node: &EventTreeNode,
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
type Violations = Vec<(String, Binding, ViolationReason)>;
type ViolationsWithoutID = Vec<(Binding, ViolationReason)>;

fn check_with_tree(
    variables: Vec<ObjectVariable>,
    nodes: Vec<TreeNode>,
    ocel: &OCEL,
) -> (Vec<usize>, Vec<ViolationsWithoutID>) {
    let linked_ocel = link_ocel_info(ocel);
    let mut binding_sizes: Vec<usize> = Vec::new();
    let mut violation_sizes: Vec<usize> = Vec::new();
    let mut violations: Vec<Violations> = Vec::new();
    let mut violations_per_node: HashMap<String, ViolationsWithoutID> =
        nodes.iter().map(|n| (n.id.clone(), Vec::new())).collect();
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
        binding_sizes.push(bindings.len());
        // for node in &nodes {
        if let Some(node) = nodes.first() {
            let (new_violations, new_bindings) =
                evaluate_tree_node(bindings, node, &linked_ocel, &nodes);
            binding_sizes.push(new_bindings.len());
            violation_sizes.push(violations.len());
            bindings = new_bindings;
            violations.extend(new_violations);
        }
        for (node_id, binding, reason) in violations.into_iter().flatten() {
            violations_per_node
                .entry(node_id)
                .or_default()
                .push((binding, reason));
            // violations_per_node.get_mut(&node_id).unwrap_or(todo!("Could not get {:?} in {:?}",node_id,violations_per_node)).push((binding,reason));
        }
        //  violations
        //     .into_iter()
        //     .flatten()
        //     .map(|(node_id, binding, reason)| (node_id, (binding, reason)))
        //     .collect();
        // }
    } else {
        println!("Finished with check (nothing to do)");
    }
    println!("No connected node left!");
    println!("Finished in {:?}", now.elapsed());

    (
        binding_sizes,
        nodes
            .iter()
            .map(|n| violations_per_node.get(&n.id).unwrap().clone())
            .collect_vec(),
    )
}

pub fn evaluate_tree_node(
    bindings: Bindings,
    node: &TreeNode,
    linked_ocel: &LinkedOCEL,
    nodes: &[TreeNode],
) -> (Vec<Violations>, Bindings) {
    println!("Evaluating Tree Node {}", node.id);
    match &node.data {
        TreeNodeType::Event(ev_tree_node) => {
            let x: Vec<(Binding, Option<ViolationReason>)> =
                match_and_add_new_bindings(Some(bindings), ev_tree_node, linked_ocel);
            let mut new_bindings: Bindings = Vec::new();
            let mut new_violations: Violations = Vec::new();
            for (b, v) in x {
                match v {
                    Some(violation) => {
                        new_violations.push((node.id.clone(), (b), violation));
                    }
                    None => {
                        new_bindings.push(b);
                    }
                }
            }
            let mut all_violations: Vec<Violations> = vec![new_violations];
            // Respect order given by passed treenode slice!
            nodes.iter().filter_map(|n| ev_tree_node.children.iter().find(|c| n.id == c.id)).for_each(|c| {
                let c_node = nodes
                    .iter()
                    .find(|n| match &n.data {
                        TreeNodeType::Event(ev) => ev.id == c.id,
                        TreeNodeType::OR(_, _) => false,
                    })
                    .unwrap();

                let (new_violations, _new_new_bindings) =
                    evaluate_tree_node(new_bindings.clone(), c_node, linked_ocel, nodes);
                // Uhh, I like that we can _not_ do that (e.g., keep parent bindings here)
                // new_bindings = new_new_bindings;
                all_violations.extend(new_violations);
            });
            (all_violations, new_bindings)
        }
        TreeNodeType::OR(node_1, node_2) => {
            let res_1: Vec<_> = bindings
                .par_iter()
                .map(|b| evaluate_tree_node(vec![b.clone()], node_1, linked_ocel, nodes))
                .collect();
            let res_2: Vec<_> = bindings
                .par_iter()
                .map(|b| evaluate_tree_node(vec![b.clone()], node_2, linked_ocel, nodes))
                .collect();

            let mut all_violations: Vec<Violations> = vec![vec![]];
            for ((i, (v1, _b1)), (v2, _b2)) in res_1.into_iter().enumerate().zip(res_2) {
                let left_sat = v1.iter().all(|inner_v| inner_v.is_empty());
                let right_sat = v2.iter().all(|inner_v| inner_v.is_empty());
                all_violations.extend(v1);
                all_violations.extend(v2);
                if left_sat || right_sat {
                } else {
                    all_violations[0].push((
                        node.id.clone(),
                        bindings[i].clone(),
                        ViolationReason::TooFewMatchingEvents,
                    ));
                }
            }
            (all_violations, bindings)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CheckWithTreeRequest {
    pub variables: Vec<ObjectVariable>,
    #[serde(rename = "nodesOrder")]
    pub nodes_order: Vec<EventTreeNode>,
}
pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(req): Json<CheckWithTreeRequest>,
) -> (
    StatusCode,
    Json<Option<(Vec<usize>, Vec<ViolationsWithoutID>)>>,
) {
    // TODO: This is just a quick shim to test ORs out:
    if req.nodes_order.len() >= 2 && req.nodes_order.len() < 4 {
        println!("[!!!] Shimming OR!");
        let mut nodes = vec![TreeNode {
            id: "or-node-id".to_string(),
            data: TreeNodeType::OR(
                Box::new(TreeNode {
                    id: req.nodes_order[0].id.clone(),
                    data: TreeNodeType::Event(req.nodes_order[0].clone()),
                }),
                Box::new(TreeNode {
                    id: req.nodes_order[1].id.clone(),
                    data: TreeNodeType::Event(req.nodes_order[1].clone()),
                }),
            ),
        }];
        nodes.extend(req.nodes_order[0..].iter().map(|n| TreeNode {
            id: n.id.clone(),
            data: TreeNodeType::Event(n.clone()),
        }));
        with_ocel_from_state(&state, |ocel| {
            (
                StatusCode::OK,
                Json(Some(check_with_tree(req.variables, nodes, ocel))),
            )
        })
        .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
    } else {
        with_ocel_from_state(&state, |ocel| {
            (
                StatusCode::OK,
                Json(Some(check_with_tree(
                    req.variables,
                    req.nodes_order
                        .into_iter()
                        .map(|n| TreeNode {
                            id: n.id.clone(),
                            data: TreeNodeType::Event(n.clone()),
                        })
                        .collect(),
                    ocel,
                ))),
            )
        })
        .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
    }
}
