use core::f32;
use std::{
    collections::{HashMap, HashSet},
};

use advanced::{EventOrObjectType};
use graph_discovery::{discover_count_constraints, discover_ef_constraints, discover_or_constraints_new};
use itertools::Itertools;

use rayon::iter::{ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    binding_box::{
        BindingBoxTree,
    },
    preprocessing::{
        linked_ocel::{IndexLinkedOCEL},
    },
};

// use self::evaluation::{get_count_constraint_fraction, get_ef_constraint_fraction};

pub mod advanced;
pub mod evaluation;
pub mod graph_discovery;

pub static SAMPLE_MIN_NUM_INSTANCES: usize = 1000;
pub static SAMPLE_FRAC: f32 = 0.1;
pub static RNG_SEED: u64 = 13375050;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventuallyFollowsConstraints {
    pub min_seconds: f64,
    pub max_seconds: f64,
    pub object_types: Vec<String>,
    pub from_event_type: String,
    pub to_event_type: String,
}

#[derive(Debug)]
pub struct EFConstraintInfo {
    pub constraint: EventuallyFollowsConstraints,
    pub supporting_object_ids: HashSet<String>,
    pub cover_fraction: f32,
}
impl EventuallyFollowsConstraints {
    fn get_constraint_name(&self) -> String {
        format!(
            "{} -> {} for {}",
            self.from_event_type,
            self.to_event_type,
            self.object_types.join(", "),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CountConstraintOptions {
    pub object_types: Vec<String>,
    pub cover_fraction: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventuallyFollowsConstraintOptions {
    pub object_types: Vec<String>,
    pub cover_fraction: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ORConstraintOptions {
    pub object_types: Vec<String>,
    pub cover_fraction: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoDiscoverConstraintsRequest {
    pub count_constraints: Option<CountConstraintOptions>,
    pub eventually_follows_constraints: Option<EventuallyFollowsConstraintOptions>,
    pub or_constraints: Option<ORConstraintOptions>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoDiscoverConstraintsResponse {
    pub constraints: Vec<(String, BindingBoxTree)>,
}

pub fn auto_discover_constraints_with_options(
    ocel: &IndexLinkedOCEL,
    options: AutoDiscoverConstraintsRequest,
) -> AutoDiscoverConstraintsResponse {
    let mut trees_per_type: HashMap<EventOrObjectType, Vec<BindingBoxTree>> = HashMap::new();
    let mut ret = AutoDiscoverConstraintsResponse {
        constraints: Vec::new(),
    };
    if let Some(eventually_follows_options) = options.eventually_follows_constraints {
        for ot in &eventually_follows_options.object_types {
            for c in discover_ef_constraints(ocel, eventually_follows_options.cover_fraction, ot) {
                ret.constraints
                    .push((c.get_constraint_name(), c.get_full_tree()));
                trees_per_type
                    .entry(EventOrObjectType::Object(ot.clone()))
                    .or_default()
                    .push(c.to_subtree("X".to_string(), 0, 2, 3))
            }
        }
    };
    if let Some(count_opts) = &options.count_constraints {
        let types = count_opts
            .object_types
            .iter()
            .map(|ot| EventOrObjectType::Object(ot.clone()))
            .collect_vec();
        for t in types {
            for cc in discover_count_constraints(ocel, count_opts.cover_fraction, t.clone()) {
                ret.constraints
                    .push((cc.get_constraint_name(), cc.get_full_tree()));
                
                    trees_per_type
                    .entry(t.clone())
                    .or_default()
                    .push(cc.to_subtree("X".to_string(), 0, 2));
            
        }
    }
}
    if let Some(or_constraint_option) = options.or_constraints {
        for ot in or_constraint_option.object_types {
            for or_c in discover_or_constraints_new(ocel, &ot,or_constraint_option.cover_fraction) {
                ret.constraints.push(or_c);
            }
        }
    }

    ret
}
