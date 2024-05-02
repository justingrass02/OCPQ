pub mod structs;

pub mod step_order;

pub mod expand_step;

#[cfg(test)]
pub mod test;

use std::time::Instant;

pub use structs::{BindingBox, BindingBoxTree, BindingStep};

use crate::preprocessing::{linked_ocel::IndexLinkedOCEL};

use structs::EvaluationResults;

pub fn evaluate_box_tree(tree: BindingBoxTree, ocel: &IndexLinkedOCEL) -> EvaluationResults {
    let now = Instant::now();
    let res = tree.evaluate(&ocel);
    println!("Evaluated in {:?} (Size: {})", now.elapsed(), res.len());
    res
}
