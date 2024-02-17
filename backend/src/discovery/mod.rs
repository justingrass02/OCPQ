use core::f32;
use std::collections::{HashMap, HashSet};

use axum::{extract::State, Json};
use itertools::Itertools;
use process_mining::OCEL;
use serde::{Deserialize, Serialize};

use crate::{
    constraints::{CountConstraint, EventType},
    ocel_qualifiers::qualifiers::get_qualifiers_for_event_types,
    preprocessing::preprocess::link_ocel_info,
    with_ocel_from_state, AppState,
};

#[derive(Serialize, Deserialize, Debug)]

#[serde(rename_all = "camelCase")]
pub struct SimpleDiscoveredCountConstraints {
    pub count_constraint: CountConstraint,
    pub object_type: String,
    pub event_type: EventType,
}
pub fn auto_discover_count_constraints(
    ocel: &OCEL,
    options: CountConstraintOptions,
) -> Vec<SimpleDiscoveredCountConstraints> {
    let linked_ocel = link_ocel_info(ocel);
    // (object_type,event_type)
    let mut num_evs_per_obj_and_ev_type: HashMap<(String, String), Vec<f32>> = HashMap::new();
    let qual_per_event_type = get_qualifiers_for_event_types(ocel);
    let obj_types_per_ev_type: HashMap<String, HashSet<String>> = ocel
        .event_types
        .iter()
        .map(|et| {
            let set: HashSet<String> = match qual_per_event_type.get(&et.name) {
                Some(hs) => hs.values().flat_map(|v| v.object_types.clone()).collect(),
                None => HashSet::new(),
            };
            (et.name.clone(), set)
        })
        .collect();
    let event_types_per_obj_type: HashMap<String, Vec<&String>> = ocel
        .object_types
        .iter()
        .map(|ot| {
            (
                ot.name.clone(),
                ocel.event_types
                    .iter()
                    .map(|et| &et.name)
                    .filter(|et| obj_types_per_ev_type.get(*et).unwrap().contains(&ot.name))
                    .collect_vec(),
            )
        })
        .collect();
    // event type, object id
    let mut map: HashMap<(&String, &String), usize> = HashMap::new();
    for object in &ocel.objects {
        for ev_type in event_types_per_obj_type.get(&object.object_type).unwrap() {
            map.insert((ev_type, &object.id), 0);
        }
    }
    println!("Init empty map!");
    for ev in &ocel.events {
        for obj_id in ev
            .relationships
            .iter()
            .flatten()
            .map(|e| &e.object_id)
            .sorted()
            .dedup()
        {
            *map.entry((&ev.event_type, obj_id)).or_default() += 1;
        }
    }
    for obj_type in &ocel.object_types {
        let evt_types = event_types_per_obj_type.get(&obj_type.name).unwrap();
        for evt_type in evt_types {
            let mut counts: Vec<f32> = Vec::new();
            for obj in linked_ocel.objects_of_type.get(&obj_type.name).unwrap() {
                counts.push(*map.get(&(evt_type, &obj.id)).unwrap() as f32);
            }
            num_evs_per_obj_and_ev_type
                .insert((obj_type.name.clone(), (*evt_type).clone()), counts);
        }
    }

    let mut ret: Vec<SimpleDiscoveredCountConstraints> = Vec::new();
    for ((object_type, event_type), counts) in num_evs_per_obj_and_ev_type {
        let mean = counts.iter().sum::<f32>() / counts.len() as f32;
        let std_deviation = counts
            .iter()
            .map(|c| {
                let diff = mean - *c;
                diff * diff
            })
            .sum::<f32>()
            .sqrt();
        let mut std_dev_factor = 0.05;
        while (counts
            .iter()
            .filter(|c| {
                (mean - std_dev_factor * std_deviation).round() <= **c
                    && **c <= (mean + std_dev_factor * std_deviation).round()
            })
            .count() as f32)
            < options.cover_fraction * counts.len() as f32
        {
            std_dev_factor += 0.05;
        }
        let min = (mean - std_dev_factor * std_deviation).round() as usize;
        let max = (mean + std_dev_factor * std_deviation).round() as usize;

        // For now, do not discover constraints with huge range; Those are most of the time not desired
        if max - min > 25 || max > 100 {
            continue;
        }
        let new_simple_count_constr = SimpleDiscoveredCountConstraints {
            count_constraint: CountConstraint { min, max },
            object_type,
            event_type: EventType::Exactly { value: event_type },
        };
        println!("New constraint: {:?}", new_simple_count_constr);
        ret.push(new_simple_count_constr)
    }
    ret
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CountConstraintOptions {
    pub cover_fraction: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoDiscoverConstraintsRequest {
    pub count_constraints: Option<CountConstraintOptions>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoDiscoverConstraintsResponse {
    pub count_constraints: Vec<SimpleDiscoveredCountConstraints>,
}

pub async fn auto_discover_constraints_handler(
    state: State<AppState>,
    Json(req): Json<AutoDiscoverConstraintsRequest>,
) -> Json<Option<AutoDiscoverConstraintsResponse>> {
    Json(with_ocel_from_state(&state, |ocel| {
        let count_constraints = match req.count_constraints {
            Some(count_options) => auto_discover_count_constraints(ocel, count_options),
            None => Vec::new(),
        };
        AutoDiscoverConstraintsResponse { count_constraints }
    }))
}
