// Idea: given >1 (arbitrary) binding box subtrees where initial binding steps are missing
// with the same input bindings (or at least a common subset?) use sampling of bindings to evaluate the binding subtrees
// Detect patterns (e.g., OR) based on the boolean results for the sampled bindings (i..e., if the subtree is satisfied for the binding)

use std::cmp::max;

use itertools::Itertools;

use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};


use crate::{
    binding_box::{structs::Variable, Binding, BindingBoxTree},
    preprocessing::linked_ocel::IndexLinkedOCEL,
};

use super::{SimpleDiscoveredCountConstraints, RNG_SEED, SAMPLE_FRAC, SAMPLE_MIN_NUM_INSTANCES};

// 1st Step: Allow building of  (simple) sampled bindings based on object/event type
pub fn generate_sample_bindings(
    ocel: &IndexLinkedOCEL,
    ocel_types: &Vec<String>,
    target_variable: Variable,
) -> Vec<Binding> {
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    match target_variable {
        Variable::Event(ev) => {
            let instances: Vec<_> = ocel_types
                .iter()
                .flat_map(|t| ocel.events_of_type.get(t))
                .flatten()
                .collect();
            let sample_count = if instances.len() >= SAMPLE_MIN_NUM_INSTANCES {
                (instances.len() as f32 * SAMPLE_FRAC).ceil() as usize
            } else {
                instances.len()
            };
            instances
                .iter()
                .choose_multiple(&mut rng, sample_count)
                .iter()
                .map(|i| Binding::default().expand_with_ev(ev, ***i))
                .collect()
        }
        Variable::Object(ov) => {
            let instances: Vec<_> = ocel_types
                .iter()
                .flat_map(|t| ocel.objects_of_type.get(t))
                .flatten()
                .collect();
            let sample_count = if instances.len() >= 1000 {
                (instances.len() as f32 * SAMPLE_FRAC).ceil() as usize
            } else {
                instances.len()
            };
            instances
                .iter()
                .choose_multiple(&mut rng, sample_count)
                .iter()
                .map(|i| Binding::default().expand_with_ob(ov, ***i))
                .collect()
        }
    }
}

// 2nd Step
pub fn check_tree_combinations(
    ocel: &IndexLinkedOCEL,
    subtrees: Vec<BindingBoxTree>,
    input_bindings: Vec<Binding>,
) {
    // First index: Binding index, second index: subtree index;
    // Value: true if subtree is satisfied for binding, false otherwise
    let sat_subtrees_per_binding: Vec<_> = input_bindings
        .par_iter()
        .map(|b| {
            subtrees
                .iter()
                .map(|t| {
                    let (_overall_res, root_res): (
                        Vec<(usize, Binding, Option<crate::binding_box::ViolationReason>)>,
                        Vec<(Binding, Option<crate::binding_box::ViolationReason>)>,
                    ) = t.nodes[0].evaluate(0, 0, b.clone(), t, ocel);
                    root_res.iter().any(|(_, v)| v.is_some())
                })
                .collect_vec()
        })
        .collect();

    let num_tree_sat = subtrees
        .iter()
        .enumerate()
        .map(|(t_index, _t)| {
            sat_subtrees_per_binding
                .iter()
                .filter(|r| r[t_index])
                .count()
        })
        .collect_vec();
    // Check combinations of subtrees
    for i in 0..subtrees.len() {
        for j in (i + 1)..subtrees.len() {
            let num_or_sat = sat_subtrees_per_binding
                .iter()
                .filter(|r| r[i] || r[j])
                .count();
            let good_or_frac = num_or_sat as f32 / max(num_tree_sat[i], num_tree_sat[j]) as f32;
            let good_sat_frac = num_or_sat as f32 / input_bindings.len() as f32;
            if good_sat_frac >= 0.8 && good_or_frac >= 1.1 {
                println!("Good OR candidate with {good_or_frac}");
                println!("{:#?}\n{:#?}\n\n", subtrees[i], subtrees[j]);
            }
        }
    }
}

pub fn test123(
    _ocel: &IndexLinkedOCEL,
    _ocel_types: &Vec<String>,
    _input_variable: Variable,
    _count_constraints: &Vec<SimpleDiscoveredCountConstraints>,
) {
    // let subtrees = count_constraints
    //     .iter()
    //     .enumerate()
    //     .map(|(i, cc)| cc.to_subtree("A".to_string(), input_variable.to_inner(), 1000 + i))
    //     .collect_vec();
    // println!("GOT {} subtrees", subtrees.len());
    // println!("First subtree: {:?}", subtrees[0]);
    // let bindings = generate_sample_bindings(ocel, ocel_types, input_variable);
    // println!("GOT {} bindings", bindings.len());
    // println!("First binding: {}", bindings[0]);
    // check_tree_combinations(ocel, subtrees, bindings);
}

// // TODO: This is not great
// // I think it would be better to _specifically_ discover constraints for violating/non-supporting bindings
// pub fn discover_for_input_bindings(
//     ocel: &IndexLinkedOCEL,
//     positive_bindings: Vec<Binding>,
//     negative_bindings: Vec<Binding>,
// ) -> Vec<BindingBoxTree> {
//     // Any input bindings; For now, we assume that the positive/negative bindings bind exactly the same variables
//     let b = positive_bindings
//         .first()
//         .or(negative_bindings.first())
//         .unwrap();
//     // For now, we simply use the first Event/Object Variable we see
//     let v = b
//         .event_map
//         .keys()
//         .next()
//         .map(|ev| Variable::Event(*ev))
//         .or(b.object_map.keys().next().map(|ov| Variable::Object(*ov)))
//         .unwrap();
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOrObjectType {
    Event(String),
    Object(String),
}

impl EventOrObjectType {
    pub fn inner(&self) -> &String {
        match self {
            EventOrObjectType::Event(et) => et,
            EventOrObjectType::Object(ot) => ot,
        }
    }
}
