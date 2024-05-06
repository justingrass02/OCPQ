pub mod structs;

pub mod step_order;

pub mod expand_step;

#[cfg(test)]
pub mod test;

use std::time::Instant;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
pub use structs::{Binding, BindingBox, BindingBoxTree, BindingStep, ViolationReason};
use ts_rs::TS;

use crate::preprocessing::linked_ocel::IndexLinkedOCEL;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateBoxTreeResult {
    pub evaluation_results: Vec<EvaluationResultWithCount>,
    pub object_ids: Vec<String>,
    pub event_ids: Vec<String>,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationResultWithCount {
    pub situations: Vec<(Binding, Option<ViolationReason>)>,
    pub situation_count: usize,
    pub situation_violated_count: usize,
}

pub fn evaluate_box_tree(tree: BindingBoxTree, ocel: &IndexLinkedOCEL) -> EvaluateBoxTreeResult {
    let now = Instant::now();
    let evaluation_results_flat = tree.evaluate(&ocel);
    let mut evaluation_results = tree
        .nodes
        .iter()
        .map(|_| EvaluationResultWithCount {
            situations: Vec::new(),
            situation_count: 0,
            situation_violated_count: 0,
        })
        .collect_vec();

    for (index, binding, viol) in evaluation_results_flat {
        let r = &mut evaluation_results[index];
        r.situations.push((binding, viol));
        r.situation_count += 1;
        if viol.is_some() {
            r.situation_violated_count += 1;
        }
    }
    println!(
        "Evaluated in {:?} (Size: {})",
        now.elapsed(),
        evaluation_results.len()
    );
    EvaluateBoxTreeResult {
        evaluation_results,
        object_ids: ocel.ocel.objects.iter().map(|o| o.id.clone()).collect(),
        event_ids: ocel.ocel.events.iter().map(|o| o.id.clone()).collect(),
    }
}
