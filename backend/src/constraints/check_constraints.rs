use core::panic;
use std::{collections::HashMap, f64::MAX, time::Instant};

use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use process_mining::event_log::ocel::ocel_struct::{OCELEvent, OCEL};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constraints::{
        Connection, FirstOrLastEventOfType, ObjectVariable, TimeConstraint, TreeNodeConnection,
    },
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
    prev_bindings: &'a Bindings,
    node: &'a EventTreeNode,
    node_id: &'a String,
    node_parents: &'a Vec<TreeNodeConnection>,
    linked_ocel: &'a LinkedOCEL,
) -> Vec<EvaluateBindingResult> {
    // Get all events of the corresponding event type (determined by node)
    // let events = linked_ocel.events_of_type.get(&node.event_type).unwrap();
    // let is_initial_binding = prev_bindings_opt.is_none();
    // let mut prev_bindings: Bindings = match prev_bindings_opt {
    //     Some(p) => p,
    //     None => {
    //         get_events_of_type_associated_with_objects(linked_ocel, &node.event_type, Vec::new())
    //             .iter()
    //             .map(|ev| {
    //                 (
    //                     AdditionalBindingInfo {
    //                         past_events: vec![PastEventInfo {
    //                             event_id: ev.id.clone(),
    //                             node_id: node.id.clone(),
    //                         }],
    //                     },
    //                     HashMap::new(),
    //                 )
    //             })
    //             .collect()
    //     }
    // };
    // Iterate over all bindings, updating them
    prev_bindings
        .par_iter()
        .map(|(info, binding)| {
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
        let num_matching_events = matching_events.len();
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
            for p in node_parents {
            if let Some(connection) = p.connection.as_ref() {

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
                        if (second_diff as f64) < connection.time_constraint.min_seconds
                            || (second_diff as f64) > connection.time_constraint.max_seconds
                        {
                            return false;
                        }
                    }
                    None => {
                        // Hmm weird... no matching parent found?!
                        eprintln!("No matching parent found for node ID {:?}; Past events: {:?}", p.id, info.past_events);
                    }
                        }}
            }
            true
        }).collect();
        let num_matching_events_after = matching_events.len();
        if num_matching_events_after < node.count_constraint.min {
            return EvaluateBindingResult::Violated((node_id.clone(),(info.clone(), binding.clone()),ViolationReason::TooFewMatchingEvents));
            }
            if num_matching_events_after > node.count_constraint.max {
                return EvaluateBindingResult::Violated((node_id.clone(),(info.clone(), binding.clone()),ViolationReason::TooManyMatchingEvents));
            }
            let x : Vec<_> = matching_events
                .iter()
                .flat_map(|matching_event| {
                    let mut info_cc: AdditionalBindingInfo = info.clone();
                    let binding: HashMap<String, BoundValue> = binding.clone();
                    // We now got one or more matching event(s)!
                    // We can now update the corresponding info...
                    // if !is_initial_binding {
                    info_cc.past_events.push(PastEventInfo {
                        event_id: matching_event.id.clone(),
                        node_id: node_id.clone(),
                    });
                    // }

                    let mut bindings: Vec<Binding> =
                        vec![((info_cc, binding.clone()))];
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
                                                    cloned_bind.1.insert(
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
                .collect();
            return EvaluateBindingResult::Satisfied(x);
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
    NoChildrenOfORSatisfied,
    // NoMatchingEvents,
    // MultipleMatchingEvents,
    // MatchingEvents,
}

type Violation = (String, Binding, ViolationReason);
type Violations = Vec<Violation>;
type ViolationsWithoutID = Vec<(Binding, ViolationReason)>;

fn check_with_tree(
    variables: Vec<ObjectVariable>,
    nodes: Vec<TreeNode>,
    ocel: &OCEL,
) -> (Vec<usize>, Vec<ViolationsWithoutID>) {
    let linked_ocel = link_ocel_info(ocel);
    let mut binding_sizes_per_node: HashMap<String, usize> =
        nodes.iter().map(|n| (n.id.clone(), 0)).collect();
    let mut violations: Violations = Vec::new();
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

        // TODO: Also consider disconnected nodes / components
        // if let Some(node) = nodes.first() {
        for node in nodes.iter() {
            if node.parents.len() == 0 {
                let (new_violations, bindings) =
                    evaluate_tree_node(&bindings, node, &linked_ocel, &nodes);
                bindings.into_iter().for_each(|(node_id, bs)| {
                    *binding_sizes_per_node.get_mut(&node_id).unwrap() += bs.len();
                });

                violations = new_violations.into_iter().flat_map(|v| v).collect_vec();
            }
        }
        for (node_id, binding, reason) in violations.into_iter() {
            violations_per_node
                .entry(node_id)
                .or_default()
                .push((binding, reason));
        }
    } else {
        println!("Finished with check (nothing to do)");
    }
    println!("No connected node left!");
    println!("Finished in {:?}", now.elapsed());

    (
        nodes
            .iter()
            .map(|n| *binding_sizes_per_node.get(&n.id).unwrap())
            .collect(),
        nodes
            .iter()
            .map(|n| violations_per_node.get(&n.id).unwrap().clone())
            .collect_vec(),
    )
}

pub enum EvaluateBindingResult {
    Satisfied(Bindings),
    Violated(Violation),
}

pub fn evaluate_tree_node(
    bindings: &Bindings,
    node: &TreeNode,
    linked_ocel: &LinkedOCEL,
    nodes: &[TreeNode],
    // Return list of violations per input binding (multiple Violations can occur for a single binding, e.g., for ORs)
    // + List of number of bindings per NODEID Vec<(String,usize)> OR
    // The bindings themself per NODEID (this would allow exposing them in the UI etc. think: Node that lists all matching bindings)
) -> (Vec<Vec<Violation>>, Vec<(String, Vec<Binding>)>) {
    println!("Evaluating Tree Node {}", node.id);
    match &node.data {
        TreeNodeType::Event(ev_tree_node) => {
            let x: Vec<EvaluateBindingResult> = match_and_add_new_bindings(
                &bindings,
                ev_tree_node,
                &node.id,
                &node.parents,
                linked_ocel,
            );
            // New (flattened) bindings with origin binding index: i.e., where it came from before flattening (given as usize)
            let mut new_bindings: Vec<Binding> = Vec::new();
            let mut origin_of_bindings: Vec<usize> = Vec::new();
            let mut violations: Vec<Vec<Violation>> = Vec::with_capacity(x.len());
            let mut ret_bindings: Vec<(String, Vec<Binding>)> = Vec::new();
            ret_bindings.push((node.id.clone(), bindings.clone()));
            for (i, ev_res) in x.into_iter().enumerate() {
                match ev_res {
                    EvaluateBindingResult::Satisfied(bindings) => {
                        for b in bindings {
                            new_bindings.push(b);
                            origin_of_bindings.push(i);
                        }
                        violations.push(Vec::new());
                    }
                    EvaluateBindingResult::Violated(v) => {
                        violations.push(vec![v]);
                    }
                }
            }

            // Respect order of children given by _nodes_
            nodes
                .iter()
                .filter_map(|n| node.children.iter().find(|c| n.id == c.id))
                .for_each(|c| {
                    let c_node = nodes.iter().find(|n| n.id == c.id).unwrap();
                    let (c_res, c_bindings) =
                        evaluate_tree_node(&new_bindings, c_node, linked_ocel, nodes);
                    ret_bindings.extend(c_bindings);
                    for (violation, binding_origin) in
                        c_res.into_iter().zip(origin_of_bindings.iter())
                    {
                        // Return violations of children if they encountered any
                        violations[*binding_origin].extend(violation);
                    }
                    // Uhh, I like that we can _not_ do that (e.g., keep parent bindings here without expanding when evaluating the next child)
                    // new_bindings = new_new_bindings;
                    // all_violations.extend(c_res.into_iter().filter_map(|r| match r {
                    //     EvaluateBindingResult::Satisfied(_) => None,
                    //     EvaluateBindingResult::Violated(v) => Some(v),
                    // }).collect());
                });

            (violations, ret_bindings)
            // (all_violations, new_bindings)
        }
        TreeNodeType::OR(node_1_id, node_2_id) => {
            let node_1 = nodes
                .iter()
                .find(|n| &n.id == node_1_id)
                .unwrap_or_else(|| {
                    panic!("OR node referencing non-existing node ID {}", node_1_id)
                });
            let node_2 = nodes
                .iter()
                .find(|n| &n.id == node_2_id)
                .unwrap_or_else(|| {
                    panic!("OR node referencing non-existing node ID {}", node_2_id)
                });
            // TOOD: Update evaluate_tree_node to not take ownership of bindings?
            let (violations_1, bindings_1) =
                evaluate_tree_node(&bindings, node_1, linked_ocel, nodes);
            let (violations_2, bindings_2) =
                evaluate_tree_node(&bindings, node_2, linked_ocel, nodes);

            let mut violations: Vec<Vec<Violation>> = Vec::new();
            let mut ret_bindings: Vec<(String, Vec<Binding>)> = vec![bindings_1, bindings_2]
                .into_iter()
                .flatten()
                .collect_vec();
            ret_bindings.push((node.id.clone(), bindings.clone()));
            for ((i, mut v1), v2) in violations_1.into_iter().enumerate().zip(violations_2) {
                let left_sat = v1.is_empty();
                let right_sat = v2.is_empty();

                v1.extend(v2);
                if left_sat || right_sat {
                    // i.e., STILL show violations even if OR is satisfied overall
                    // If this is not wanted, remember to also update the bindings/num. bindings propagated to the top
                    violations.push(v1);
                } else {
                    v1.push((
                        node.id.clone(),
                        (bindings[i].0.clone(), bindings[i].1.clone()),
                        ViolationReason::NoChildrenOfORSatisfied,
                    ));
                    violations.push(v1);
                    // OR:
                    // violations.push(v2);
                }
            }
            (violations, ret_bindings)
        }
    }
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
) -> (
    StatusCode,
    Json<Option<(Vec<usize>, Vec<ViolationsWithoutID>)>>,
) {
    with_ocel_from_state(&state, |ocel| {
        (
            StatusCode::OK,
            Json(Some(check_with_tree(req.variables, req.nodes_order, ocel))),
        )
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}
