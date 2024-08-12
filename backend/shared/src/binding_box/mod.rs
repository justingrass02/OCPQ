pub mod structs;

pub mod step_order;

pub mod expand_step;

#[cfg(test)]
pub mod test;

use std::{fs::File, io::BufWriter, time::Instant};

use chrono::DateTime;
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct CheckWithBoxTreeRequest {
    pub tree: BindingBoxTree,
    pub measure_performance: Option<bool>,
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

pub fn evaluate_box_tree(
    tree: BindingBoxTree,
    ocel: &IndexLinkedOCEL,
    measure_performance: bool,
) -> EvaluateBoxTreeResult {
    if measure_performance {
        let n = 10;
        let mut eval_times = Vec::new();
        let st = std::time::SystemTime::now();
        let dt: DateTime<chrono::Utc> = st.into();
        let dt_iso = dt.to_rfc3339();
        let tree_json_file = File::create(format!("{dt_iso}-tree.json")).unwrap();
        serde_json::to_writer_pretty(BufWriter::new(tree_json_file), &tree).unwrap();
        for _ in 0..n {
            let start = Instant::now();
            tree.evaluate(ocel);
            eval_times.push(start.elapsed().as_secs_f64());
        }
        let seconds_json_file = File::create(format!("{dt_iso}-durations.json")).unwrap();
        serde_json::to_writer_pretty(BufWriter::new(seconds_json_file), &eval_times).unwrap();
        println!("Evaluation time: {eval_times:?}");
    }
    let now = Instant::now();
    let evaluation_results_flat = tree.evaluate(ocel);
    println!("Tree Evaluated in {:?}", now.elapsed());
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
