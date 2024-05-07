use std::time::Instant;

use itertools::Itertools;
use process_mining::import_ocel_json_from_path;

use crate::{
    binding_box::structs::{BindingBoxTree, BindingBoxTreeNode, FilterConstraint, ViolationReason}, preprocessing::linked_ocel::link_ocel_info,
};

use super::{BindingBox, BindingStep};

#[test]
fn basic_binding_box() {
    let binding_box = BindingBox {
        new_event_vars: vec![
            (
                0.into(),
                vec!["place order".to_string()].into_iter().collect(),
            ),
            (
                1.into(),
                vec!["place order".to_string()].into_iter().collect(),
            ),
        ]
        .into_iter()
        .collect(),
        new_object_vars: vec![
            (
                0.into(),
                vec!["customers".to_string()].into_iter().collect(),
            ),
            (1.into(), vec!["orders".to_string()].into_iter().collect()),
            (2.into(), vec!["orders".to_string()].into_iter().collect()),
        ]
        .into_iter()
        .collect(),
        filter_constraint: vec![
            FilterConstraint::TimeBetweenEvents(0.into(), 1.into(), (Some(0.0000000001), None)),
            FilterConstraint::ObjectAssociatedWithEvent(1.into(), 0.into(), None),
            FilterConstraint::ObjectAssociatedWithEvent(2.into(), 1.into(), None),
            FilterConstraint::ObjectAssociatedWithObject(
                0.into(),
                1.into(),
                Some("places".to_string()),
            ),
            FilterConstraint::ObjectAssociatedWithObject(
                0.into(),
                2.into(),
                Some("places".to_string()),
            ),
        ],
    };

    let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
    let linked_ocel = link_ocel_info(ocel);
    let steps = BindingStep::get_binding_order(&binding_box);
    println!("Steps: {:?}", steps);
    let now = Instant::now();
    let res = binding_box.expand_with_steps_empty(&linked_ocel, &steps);
    println!("Output binding size: {} in {:?}", res.len(), now.elapsed());
    println!("First binding: {}", res.first().unwrap());
}

#[test]
fn connected_binding_box() {
    let binding_box = BindingBox {
        new_event_vars: vec![].into_iter().collect(),
        new_object_vars: vec![(0.into(), vec!["orders".to_string()].into_iter().collect())]
            .into_iter()
            .collect(),
        filter_constraint: vec![],
    };
    let binding_box_2 = BindingBox {
        new_event_vars: vec![(
            0.into(),
            vec!["payment reminder".to_string()].into_iter().collect(),
        )]
        .into_iter()
        .collect(),
        new_object_vars: vec![
            // (1.into(),vec!["employees".to_string()].into_iter().collect())
        ]
        .into_iter()
        .collect(),
        filter_constraint: vec![
            FilterConstraint::ObjectAssociatedWithEvent(0.into(), 0.into(), None),
            // FilterConstraint::ObjectAssociatedWithEvent(1.into(), 0.into(), None)
        ],
    };

    let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
    let linked_ocel = link_ocel_info(ocel);
    let now = Instant::now();
    println!("Steps: {:?}", BindingStep::get_binding_order(&binding_box));
    let res = binding_box.expand_empty(&linked_ocel);
    println!("Output binding size: {} in {:?}", res.len(), now.elapsed());
    println!(
        "Steps: {:?}",
        BindingStep::get_binding_order(&binding_box_2)
    );
    let (min, max) = (1, 1);
    let mut outcome = Vec::new();
    for b in res {
        let res2 = binding_box_2.expand(vec![b.clone()], &linked_ocel);
        if res2.len() >= min && res2.len() <= max {
            outcome.push((b, true));
        } else {
            outcome.push((b, false));
        }
    }
    let num_sat = outcome.iter().filter(|(_, sat)| *sat).count() as f32;
    println!(
        "Total time {:?} {}",
        now.elapsed(),
        num_sat / outcome.len() as f32
    );
}

#[test]
fn simple_binding_box_tree() {
    let bb1 = BindingBox {
        new_event_vars: vec![].into_iter().collect(),
        new_object_vars: vec![(0.into(), vec!["orders".to_string()].into_iter().collect())]
            .into_iter()
            .collect(),
        filter_constraint: vec![],
    };
    let bb2 = BindingBox {
        new_event_vars: vec![(
            0.into(),
            vec!["place order".to_string()].into_iter().collect(),
        )]
        .into_iter()
        .collect(),
        new_object_vars: vec![].into_iter().collect(),
        filter_constraint: vec![FilterConstraint::ObjectAssociatedWithEvent(
            0.into(),
            0.into(),
            None,
        )],
    };

    let bb3 = BindingBox {
        new_event_vars: vec![(
            1.into(),
            vec!["pay order".to_string()].into_iter().collect(),
        )]
        .into_iter()
        .collect(),
        new_object_vars: vec![].into_iter().collect(),
        filter_constraint: vec![
            FilterConstraint::ObjectAssociatedWithEvent(0.into(), 1.into(), None),
            FilterConstraint::TimeBetweenEvents(
                0.into(),
                1.into(),
                (Some(0.0), Some(60.0 * 60.0 * 24.0 * 7.0 * 3.0)),
            ),
        ],
    };

    let bb4 = BindingBox {
        new_event_vars: vec![(
            2.into(),
            vec!["payment reminder".to_string()].into_iter().collect(),
        )]
        .into_iter()
        .collect(),
        new_object_vars: vec![].into_iter().collect(),
        filter_constraint: vec![FilterConstraint::ObjectAssociatedWithEvent(
            0.into(),
            2.into(),
            None,
        )],
    };

    let tree = BindingBoxTree {
        nodes: vec![
            BindingBoxTreeNode::Box(bb1, vec![1]), // 0
            BindingBoxTreeNode::OR(2, 4),          // 1
            BindingBoxTreeNode::Box(bb2, vec![3]), // 2
            BindingBoxTreeNode::Box(bb3, vec![]),  // 3
            BindingBoxTreeNode::Box(bb4, vec![]),  // 4
        ],
        size_constraints: vec![((1, 4), (Some(1), None)), ((2, 3), (Some(1), None))]
            .into_iter()
            .collect(),
    };

    let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
    let linked_ocel = link_ocel_info(ocel);
    let now = Instant::now();
    let res = tree
        .evaluate(&linked_ocel)
        .into_iter()
        .filter(|(_, _, reason)| !matches!(reason, Some(ViolationReason::ChildNotSatisfied)))
        .collect_vec();
    println!("Took {:?}", now.elapsed());
    for i in 1..tree.nodes.len() {
        let total_num = res.iter().filter(|(index, _, _)| *index == i).count();
        let satisfied_num = res
            .iter()
            .filter(|(index, _, v)| v.is_none() && *index == i)
            .count();
        println!(
            "Node {i}: {} / {} satisfied (Violations: {:.2}%)",
            satisfied_num,
            total_num,
            100.0 * (total_num as f32 - satisfied_num as f32) / total_num as f32
        )
    }
}

#[test]
fn complex_binding_box_tree() {
    let bb1 = BindingBox {
        new_event_vars: vec![
            (
                0.into(),
                vec!["place order".to_string()].into_iter().collect(),
            ),
            (
                1.into(),
                vec!["place order".to_string()].into_iter().collect(),
            ),
        ]
        .into_iter()
        .collect(),
        new_object_vars: vec![
            (1.into(), vec!["orders".to_string()].into_iter().collect()),
            (2.into(), vec!["orders".to_string()].into_iter().collect()),
            (
                0.into(),
                vec!["customers".to_string()].into_iter().collect(),
            ),
        ]
        .into_iter()
        .collect(),
        filter_constraint: vec![
            FilterConstraint::ObjectAssociatedWithEvent(0.into(), 0.into(), None),
            FilterConstraint::ObjectAssociatedWithEvent(0.into(), 1.into(), None),
            FilterConstraint::ObjectAssociatedWithEvent(1.into(), 0.into(), None),
            FilterConstraint::ObjectAssociatedWithEvent(2.into(), 1.into(), None),
            FilterConstraint::ObjectAssociatedWithObject(0.into(), 1.into(), Some("places".into())),
            FilterConstraint::ObjectAssociatedWithObject(0.into(), 2.into(), Some("places".into())),
            FilterConstraint::TimeBetweenEvents(0.into(), 1.into(), (Some(0.0001), None)),
        ],
    };
    println!("Steps: {:?}", BindingStep::get_binding_order(&bb1));
    let bb2 = BindingBox {
        new_event_vars: vec![
            (
                2.into(),
                vec!["confirm order".to_string()].into_iter().collect(),
            ),
            (
                3.into(),
                vec!["confirm order".to_string()].into_iter().collect(),
            ),
        ]
        .into_iter()
        .collect(),
        new_object_vars: vec![].into_iter().collect(),
        filter_constraint: vec![
            FilterConstraint::ObjectAssociatedWithEvent(1.into(), 2.into(), None),
            FilterConstraint::ObjectAssociatedWithEvent(2.into(), 3.into(), None),
            FilterConstraint::TimeBetweenEvents(2.into(), 3.into(), (Some(0.0001), None)),
        ],
    };

    let tree = BindingBoxTree {
        nodes: vec![
            BindingBoxTreeNode::Box(bb1, vec![1]), // 0
            BindingBoxTreeNode::Box(bb2, vec![]),  // 1
        ],
        size_constraints: vec![
            ((0, 0), (Some(100), Some(100000000))),
            ((0, 1), (Some(1), Some(1))),
        ]
        .into_iter()
        .collect(),
    };

    println!("\n{}\n", serde_json::to_string_pretty(&tree).unwrap());

    let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
    let linked_ocel = link_ocel_info(ocel);
    let now = Instant::now();
    let res = tree
        .evaluate(&linked_ocel)
        .into_iter()
        .filter(|(_, _, reason)| !matches!(reason, Some(ViolationReason::ChildNotSatisfied)))
        .collect_vec();
    println!("Took {:?}", now.elapsed());
    for i in 0..tree.nodes.len() {
        let total_num = res.iter().filter(|(index, _, _)| *index == i).count();
        let satisfied_num = res
            .iter()
            .filter(|(index, _, v)| v.is_none() && *index == i)
            .count();
        println!(
            "Node {i}: {} / {} satisfied (Violations: {:.2}%)",
            satisfied_num,
            total_num,
            100.0 * (total_num as f32 - satisfied_num as f32) / total_num as f32
        )
    }
}
