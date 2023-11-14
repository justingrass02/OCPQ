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

    #[derive(Debug)]
    enum BoundValue {
        Single(String),
        Multiple(Vec<String>),
    }
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
                                DependencyType::ExistsInTarget => {
                                    binding_values.get_mut(variable_name).unwrap().extend(
                                        object_id_vals.into_iter().map(|id| BoundValue::Single(id)),
                                    );
                                }
                                DependencyType::ExistsInSource => {
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

            let complete_bindings: Vec<HashMap<&String, &BoundValue>> = evs_bindings
                .iter()
                .flat_map(|b| {
                    if b.len() > 0 {
                        b.into_iter()
                            .filter(|(_, b)| b.len() > 0)
                            .map(|(a, bs)| bs.into_iter().map(move |b| (a, b)))
                            .multi_cartesian_product()
                            .map(|x| x.into_iter().collect())
                            .collect()
                    } else {
                        Vec::new()
                    }
                })
                .collect();
            println!("Total Combinations: {}", complete_bindings.len());
        }
    }
    return Vec::new();
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
