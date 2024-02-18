use core::f32;
use std::collections::{HashMap, HashSet};

use axum::{extract::State, Json};
use itertools::Itertools;
use process_mining::OCEL;
use serde::{Deserialize, Serialize};

use crate::{
    constraints::{CountConstraint, EventType, SecondsRange},
    ocel_qualifiers::qualifiers::get_qualifiers_for_event_types,
    preprocessing::preprocess::{link_ocel_info, LinkedOCEL},
    with_ocel_from_state, AppState,
};
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventuallyFollowsConstraints {
    pub seconds_range: SecondsRange,
    pub object_types: Vec<String>,
    pub from_event_type: String,
    pub to_event_type: String,
}

pub fn auto_discover_eventually_follows(
    linked_ocel: &LinkedOCEL,
    options: EventuallyFollowsConstraintOptions,
) -> Vec<EventuallyFollowsConstraints> {
    // Prev. Event Type, Event Type, Object Type -> numSeconds delay
    let mut map: HashMap<(&String, &String, &String), Vec<i64>> = HashMap::new();
    // Event Type, Object Type -> #Encountered occurences
    let mut event_type_count_per_obj_type: HashMap<(&String, &String), usize> = HashMap::new();
    for ot in &options.object_types {
        for o in linked_ocel.objects_of_type.get(ot).unwrap_or(&vec![]) {
            if let Some(ev_ids) = linked_ocel.object_events_map.get(&o.id) {
                let ordered_events = ev_ids
                    .iter()
                    .map(|ev_id| linked_ocel.event_map.get(ev_id).unwrap())
                    .sorted_by_key(|ev| ev.time)
                    .collect_vec();
                for i in 0..ordered_events.len() {
                    let prev_ev = ordered_events[i];
                    *event_type_count_per_obj_type
                        .entry((&prev_ev.event_type, &o.object_type))
                        .or_default() += 1;
                    for j in i + 1..ordered_events.len() {
                        let next_ev = ordered_events[j];
                        if ordered_events
                            .iter()
                            .skip(i)
                            .take(j - i)
                            .any(|ev| ev.event_type == next_ev.event_type)
                        {
                            continue;
                        }
                        if next_ev.event_type == prev_ev.event_type {
                            break;
                        }
                        map.entry((&prev_ev.event_type, &next_ev.event_type, &o.object_type))
                            .or_insert(Vec::new())
                            .push((next_ev.time - prev_ev.time).num_seconds());
                    }
                }
            }
        }
    }
    let mut ret: Vec<EventuallyFollowsConstraints> = Vec::new();
    for prev_et in linked_ocel.events_of_type.keys() {
        for next_et in linked_ocel.events_of_type.keys() {
            for obj_type in &options.object_types {
                let count = *event_type_count_per_obj_type
                    .get(&(prev_et, obj_type))
                    .unwrap_or(&0);
                if count > 0 {
                    if let Some(delay_seconds) = map.get(&(prev_et, next_et, obj_type)) {
                        let fraction = delay_seconds.len() as f32 / count as f32;
                        if fraction >= options.cover_fraction {
                            let mean_delay_seconds = delay_seconds.iter().sum::<i64>() as f32
                                / delay_seconds.len() as f32;
                            let delay_seconds_std_deviation = delay_seconds
                                .iter()
                                .map(|c| {
                                    let diff = mean_delay_seconds - *c as f32;
                                    diff * diff
                                })
                                .sum::<f32>()
                                .sqrt();
                            let mut std_dev_factor: f32 = 0.001;
                            while (delay_seconds
                                .iter()
                                .filter(|c| {
                                    (mean_delay_seconds
                                        - std_dev_factor * delay_seconds_std_deviation)
                                        <= **c as f32
                                        && **c as f32
                                            <= (mean_delay_seconds
                                                + std_dev_factor * delay_seconds_std_deviation)
                                })
                                .count() as f32)
                                < options.cover_fraction * delay_seconds.len() as f32
                            {
                                std_dev_factor += 0.001;
                            }
                            let min =
                                mean_delay_seconds - std_dev_factor * delay_seconds_std_deviation;
                            let max =
                                mean_delay_seconds + std_dev_factor * delay_seconds_std_deviation;

                            ret.push(EventuallyFollowsConstraints {
                                seconds_range: SecondsRange {
                                    min_seconds: min.max(0.0) as f64,
                                    max_seconds: max as f64,
                                },
                                object_types: vec![obj_type.clone()],
                                from_event_type: prev_et.clone(),
                                to_event_type: next_et.clone(),
                            });
                            println!(
                                "{:.2} {} -> {} for ot {} mean: {:.2} ; {:.2}-{:.2} ",
                                fraction,
                                prev_et,
                                next_et,
                                obj_type,
                                mean_delay_seconds,
                                min / (60.0 * 60.0),
                                max / (60.0 * 60.0),
                            );
                        }
                    }
                }
            }
        }
    }
    ret
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimpleDiscoveredCountConstraints {
    pub count_constraint: CountConstraint,
    pub object_type: String,
    pub event_type: EventType,
}
pub fn auto_discover_count_constraints(
    ocel: &OCEL,
    linked_ocel: &LinkedOCEL,
    options: CountConstraintOptions,
) -> Vec<SimpleDiscoveredCountConstraints> {
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
    let event_types_per_obj_type: HashMap<String, Vec<&String>> = options
        .object_types
        .iter()
        .map(|ot| {
            (
                ot.clone(),
                ocel.event_types
                    .iter()
                    .map(|et| &et.name)
                    .filter(|et| obj_types_per_ev_type.get(*et).unwrap().contains(ot))
                    .collect_vec(),
            )
        })
        .collect();
    // event type, object id
    let mut map: HashMap<(&String, &String), usize> = HashMap::new();
    for object_type in &options.object_types {
        for object in linked_ocel
            .objects_of_type
            .get(object_type)
            .unwrap_or(&Vec::new())
        {
            for ev_type in event_types_per_obj_type.get(&object.object_type).unwrap() {
                map.insert((ev_type, &object.id), 0);
            }
        }
    }
    for ev in &ocel.events {
        for obj_id in ev
            .relationships
            .iter()
            .flatten()
            .map(|e| &e.object_id)
            .filter(|o| {
                options
                    .object_types
                    .contains(&linked_ocel.object_map.get(*o).unwrap().object_type)
            })
            .sorted()
            .dedup()
        {
            *map.entry((&ev.event_type, obj_id)).or_default() += 1;
        }
    }
    for obj_type in &options.object_types {
        let evt_types = event_types_per_obj_type.get(obj_type).unwrap();
        for evt_type in evt_types {
            let mut counts: Vec<f32> = Vec::new();
            for obj in linked_ocel.objects_of_type.get(obj_type).unwrap() {
                counts.push(*map.get(&(evt_type, &obj.id)).unwrap() as f32);
            }
            num_evs_per_obj_and_ev_type.insert((obj_type.clone(), (*evt_type).clone()), counts);
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
        let mut std_dev_factor = 0.001;
        while (counts
            .iter()
            .filter(|c| {
                (mean - std_dev_factor * std_deviation).round() <= **c
                    && **c <= (mean + std_dev_factor * std_deviation).round()
            })
            .count() as f32)
            < options.cover_fraction * counts.len() as f32
        {
            std_dev_factor += 0.001;
        }
        let min = (mean - std_dev_factor * std_deviation).round() as usize;
        let max = (mean + std_dev_factor * std_deviation).round() as usize;

        // For now, do not discover constraints with huge range; Those are most of the time not desired
        // if max - min > 25 || max > 100 {
        //     continue;
        // }
        let new_simple_count_constr = SimpleDiscoveredCountConstraints {
            count_constraint: CountConstraint { min, max },
            object_type,
            event_type: EventType::Exactly { value: event_type },
        };
        ret.push(new_simple_count_constr)
    }
    ret
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
pub struct AutoDiscoverConstraintsRequest {
    pub count_constraints: Option<CountConstraintOptions>,
    pub eventually_follows_constraints: Option<EventuallyFollowsConstraintOptions>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoDiscoverConstraintsResponse {
    pub count_constraints: Vec<SimpleDiscoveredCountConstraints>,
    pub eventually_follows_constraints: Vec<EventuallyFollowsConstraints>,
}

pub async fn auto_discover_constraints_handler(
    state: State<AppState>,
    Json(req): Json<AutoDiscoverConstraintsRequest>,
) -> Json<Option<AutoDiscoverConstraintsResponse>> {
    Json(with_ocel_from_state(&state, |ocel| {
        let linked_ocel = link_ocel_info(ocel);
        let count_constraints = match req.count_constraints {
            Some(count_options) => {
                auto_discover_count_constraints(ocel, &linked_ocel, count_options)
            }
            None => Vec::new(),
        };
        let eventually_follows_constraints = match req.eventually_follows_constraints {
            Some(eventually_follows_options) => {
                auto_discover_eventually_follows(&linked_ocel, eventually_follows_options)
            }
            None => Vec::new(),
        };
        AutoDiscoverConstraintsResponse {
            count_constraints,
            eventually_follows_constraints,
        }
    }))
}
