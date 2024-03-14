use core::f32;
use std::{
    collections::{HashMap, HashSet},
    f64::MAX,
};

use axum::{extract::State, Json};
use itertools::Itertools;
use process_mining::{OCEL};
use serde::{Deserialize, Serialize};

use crate::{
    constraints::{CountConstraint, EventType, SecondsRange},
    preprocessing::preprocess::{link_ocel_info, LinkedOCEL},
    with_ocel_from_state, AppState,
};

use self::evaluation::{get_count_constraint_fraction, get_ef_constraint_fraction};

pub mod evaluation;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventuallyFollowsConstraints {
    pub seconds_range: SecondsRange,
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

pub fn auto_discover_eventually_follows(
    linked_ocel: &LinkedOCEL,
    object_ids: Option<HashSet<String>>,
    options: EventuallyFollowsConstraintOptions,
) -> Vec<EFConstraintInfo> {
    let object_ids = object_ids.as_ref();
    // Prev. Event Type, Event Type, Object Type -> Object ID numSeconds delay
    let mut map: HashMap<(&String, &String, &String), Vec<(String, i64)>> = HashMap::new();
    // Same as above but -> (Prev. Event ID, Event ID)
    let mut ev_map: HashMap<(&String, &String, &String), HashSet<(&String, &String)>> =
        HashMap::new();
    // Event Type, Object Type -> #Encountered occurences
    let mut event_type_count_per_obj_type: HashMap<(&String, &String), usize> = HashMap::new();
    for ot in &options.object_types {
        for o in linked_ocel.objects_of_type.get(ot).unwrap_or(&vec![]) {
            if object_ids.is_none() || object_ids.unwrap().contains(&o.id) {
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
                                .or_default()
                                .push((
                                    o.id.clone(),
                                    ((next_ev.time - prev_ev.time).num_seconds()),
                                ));

                            ev_map
                                .entry((&prev_ev.event_type, &next_ev.event_type, &o.object_type))
                                .or_default()
                                .insert((&prev_ev.id, &next_ev.id));
                        }
                    }
                }
            }
        }
    }

    let mut ret: Vec<EFConstraintInfo> = Vec::new();
    for prev_et in linked_ocel.events_of_type.keys() {
        for next_et in linked_ocel.events_of_type.keys() {
            let common_obj_types: HashSet<Vec<_>> = options
                .object_types
                .iter()
                .filter_map(|obj_type| {
                    let evs = ev_map.get(&(prev_et, next_et, obj_type));
                    match evs {
                        Some(_evs) => {
                            // let mut other_obj_types_with_same_evs: HashSet<&String> = options
                            //     .object_types
                            //     .iter()
                            //     .filter(|obj_type2| {
                            //         ev_map
                            //             .get(&(prev_et, next_et, obj_type2))
                            //             .and_then(|evs2| Some(evs2.is_superset(evs)))
                            //             .is_some_and(|b| b)
                            //     })
                            //     .collect();
                            // ↓ Disables merging of object types
                            let mut other_obj_types_with_same_evs = HashSet::new();
                            other_obj_types_with_same_evs.insert(obj_type);
                            Some(
                                other_obj_types_with_same_evs
                                    .into_iter()
                                    .sorted()
                                    .collect_vec(),
                            )
                        }
                        None => None,
                    }
                })
                .collect();
            // if common_obj_types.len() > 0 {
            //     println!("{prev_et} -> {next_et}: {:?}", common_obj_types);
            // }
            //     let mut ev_sets: Vec<_> = options.object_types.iter().flat_map(|obj_type| match ev_map.get(&(prev_et,next_et,obj_type)) {
            //         Some(evts) => evts.into_iter().map(|evs| (obj_type,evs)).collect(),
            //         None => vec![],
            //     }

            // ).collect();
            // ev_sets.iter().map(|(obj_type,(prev_ev,next_ev)))

            for obj_types in common_obj_types {
                if obj_types.is_empty() {
                    eprintln!("obj_types of length 0");
                    continue;
                }
                let obj_type = obj_types[0];
                let count = *event_type_count_per_obj_type
                    .get(&(prev_et, obj_type))
                    .unwrap_or(&0);
                if count > 0 {
                    if let Some(delay_seconds) = map.get(&(prev_et, next_et, obj_type)) {
                        let fraction = delay_seconds.len() as f32 / count as f32;
                        if fraction >= options.cover_fraction {
                            let mean_delay_seconds =
                                delay_seconds.iter().map(|(_, c)| c).sum::<i64>() as f64
                                    / delay_seconds.len() as f64;
                            let delay_seconds_std_deviation = delay_seconds
                                .iter()
                                .map(|(_, c)| {
                                    let diff = mean_delay_seconds - *c as f64;
                                    diff * diff
                                })
                                .sum::<f64>()
                                .sqrt();
                            let mut std_dev_factor: f64 = 0.003;
                            let mut constraint = EventuallyFollowsConstraints {
                                seconds_range: SecondsRange {
                                    min_seconds: 0.0,
                                    max_seconds: MAX,
                                },
                                object_types: obj_types.into_iter().cloned().collect_vec(),
                                from_event_type: prev_et.clone(),
                                to_event_type: next_et.clone(),
                            };
                            let rel_object_ids = match object_ids {
                                // Sad that we clone here
                                // TODO: look into changing
                                Some(obj_ids) => obj_ids.clone(),
                                None => {
                                    let x: HashSet<String> = linked_ocel
                                        .objects_of_type
                                        .get(obj_type)
                                        .unwrap()
                                        .iter()
                                        .map(|obj| obj.id.clone())
                                        .collect();
                                    x
                                }
                            };
                            let max_achievable = get_ef_constraint_fraction(
                                linked_ocel,
                                &constraint,
                                &rel_object_ids,
                                false,
                            )
                            .0;
                            if max_achievable < options.cover_fraction {
                                if prev_et == "place order" && next_et == "pay order" {
                                    println!("!!!! {} {}", max_achievable, options.cover_fraction);
                                }
                                continue;
                            }

                            constraint.seconds_range = SecondsRange {
                                min_seconds: mean_delay_seconds, //TODO: Re-eanble mean_delay_seconds,
                                max_seconds: mean_delay_seconds, // mean_delay_seconds,
                            };

                            while get_ef_constraint_fraction(
                                linked_ocel,
                                &constraint,
                                &rel_object_ids,
                                false,
                            )
                            .0 < options.cover_fraction
                            {
                                std_dev_factor += 0.01;
                                // TODO: Re-enable
                                constraint.seconds_range.min_seconds = mean_delay_seconds
                                    - std_dev_factor * delay_seconds_std_deviation;
                                constraint.seconds_range.max_seconds = mean_delay_seconds
                                    + std_dev_factor * delay_seconds_std_deviation;
                            }
                            // Min should be >= 0.0
                            constraint.seconds_range.min_seconds =
                                constraint.seconds_range.min_seconds.max(0.0);

                            let (cover_fraction, supporting_object_ids) =
                                get_ef_constraint_fraction(
                                    linked_ocel,
                                    &constraint,
                                    &rel_object_ids,
                                    true,
                                );

                            ret.push(EFConstraintInfo {
                                constraint,
                                cover_fraction,
                                supporting_object_ids: supporting_object_ids.unwrap(),
                            });
                            // println!(
                            //     "{:.2} {} -> {} for ot {} mean: {:.2} ; {:.2}-{:.2} ",
                            //     fraction,
                            //     prev_et,
                            //     next_et,
                            //     obj_type,
                            //     mean_delay_seconds,
                            //     min / (60.0 * 60.0),
                            //     max / (60.0 * 60.0),
                            // );
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
// We might want to also return a set of "supporting objects" for each discovered constraints
// These are the objects for which the count constraint is satisfied
// This would allows us to build constraints specifically for the _same_ or the _other_ (i.e., set of objects of same type not supported)
// Would be useful for constructing/discovering targeted OR (or XOR) constraints

// Similiarly, it would be nice to have some sort of input object_ids (only those should be considered)

#[derive(Debug)]
pub struct CountConstraintInfo {
    pub constraint: SimpleDiscoveredCountConstraints,
    pub supporting_object_ids: HashSet<String>,
    pub cover_fraction: f32,
}

pub fn get_obj_types_per_ev_type<'a>(
    linked_ocel: &'a LinkedOCEL,
) -> HashMap<&'a String, HashSet<&'a String>> {
    // let qual_per_event_type = get_qualifiers_for_event_types(ocel);
    let mut obj_types_per_ev_type: HashMap<&String, HashSet<&String>> = HashMap::new();
    for ev in linked_ocel.event_map.values() {
        if let Some(rels) = &ev.relationships {
            for r in rels {
                if let Some(obj) = linked_ocel.object_map.get(&r.object_id) {
                    obj_types_per_ev_type
                        .entry(&ev.event_type)
                        .or_default()
                        .insert(&obj.object_type);
                }
            }
        }
    }
    // let obj_types_per_ev_type: HashMap<String, HashSet<String>> = ocel
    //     .event_types
    //     .iter()
    //     .map(|et| {
    //         let set: HashSet<String> = match qual_per_event_type.get(&et.name) {
    //             Some(hs) => hs.values().flat_map(|v| v.object_types.clone()).collect(),
    //             None => HashSet::new(),
    //         };
    //         (et.name.clone(), set)
    //     })
    //     .collect();
    obj_types_per_ev_type
}
pub fn auto_discover_count_constraints(
    ocel: &OCEL,
    obj_types_per_ev_type: &HashMap<&String, HashSet<&String>>,
    linked_ocel: &LinkedOCEL,
    object_ids: Option<HashSet<String>>,
    options: CountConstraintOptions,
    // Constraint + Supporting objects
) -> Vec<CountConstraintInfo> {
    let mut num_evs_per_obj_and_ev_type: HashMap<(String, String), Vec<(f32, String)>> =
        HashMap::new();
    let object_ids = object_ids.as_ref();

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
            if object_ids.is_none() || object_ids.unwrap().contains(&object.id) {
                for ev_type in event_types_per_obj_type.get(&object.object_type).unwrap() {
                    map.insert((ev_type, &object.id), 0);
                }
            }
        }
    }
    for ev in &ocel.events {
        for obj_id in ev
            .relationships
            .iter()
            .flatten()
            .map(|e| &e.object_id)
            .filter(|o_id| {
                if object_ids.is_none() || object_ids.unwrap().contains(&o_id.to_string()) {
                    match linked_ocel.object_map.get(*o_id) {
                        Some(o) => options.object_types.contains(&o.object_type),
                        None => false,
                    }
                } else {
                    false
                }
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
            let mut counts: Vec<(f32, String)> = Vec::new();
            for obj in linked_ocel.objects_of_type.get(obj_type).unwrap() {
                if object_ids.is_none() || object_ids.unwrap().contains(&obj.id) {
                    counts.push((
                        *map.get(&(evt_type, &obj.id)).unwrap() as f32,
                        obj.id.clone(),
                    ));
                }
            }
            num_evs_per_obj_and_ev_type.insert((obj_type.clone(), (*evt_type).clone()), counts);
        }
    }

    let mut ret: Vec<CountConstraintInfo> = Vec::new();
    for ((object_type, event_type), counts) in num_evs_per_obj_and_ev_type {
        let rel_object_ids = match object_ids {
            // Sad that we clone here
            // TODO: look into changing
            Some(obj_ids) => obj_ids.clone(),
            None => {
                let x: HashSet<String> = linked_ocel
                    .objects_of_type
                    .get(&object_type)
                    .unwrap()
                    .iter()
                    .map(|obj| obj.id.clone())
                    .collect();
                x
            }
        };

        let mean = counts.iter().map(|(c, _)| c).sum::<f32>() / counts.len() as f32;
        let std_deviation = counts
            .iter()
            .map(|(c, _)| {
                let diff = mean - *c;
                diff * diff
            })
            .sum::<f32>()
            .sqrt();
        let mut std_dev_factor = 0.003;
        let mut constraint = SimpleDiscoveredCountConstraints {
            count_constraint: CountConstraint {
                min: mean.round() as usize,
                max: mean.round() as usize,
            },
            object_type,
            event_type: EventType::Exactly { value: event_type },
        };

        while get_count_constraint_fraction(linked_ocel, &constraint, &rel_object_ids, false).0
            < options.cover_fraction
        {
            std_dev_factor += 0.003;
            constraint.count_constraint = CountConstraint {
                min: (mean - std_dev_factor * std_deviation).round() as usize,
                max: (mean + std_dev_factor * std_deviation).round() as usize,
            }
        }

        // For now, do not discover constraints with huge range; Those are most of the time not desired
        if constraint.count_constraint.max - constraint.count_constraint.min > 25
            || constraint.count_constraint.max > 100
        {
            continue;
        } else {
            let (cover_fraction, supporting_object_ids) =
                get_count_constraint_fraction(linked_ocel, &constraint, &rel_object_ids, true);
            ret.push(CountConstraintInfo {
                constraint,
                supporting_object_ids: supporting_object_ids.unwrap(),
                cover_fraction,
            })
        }
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
        let obj_types_per_ev_type = get_obj_types_per_ev_type(&linked_ocel);
        let count_constraints = match req.count_constraints {
            Some(count_options) => auto_discover_count_constraints(
                ocel,
                &obj_types_per_ev_type,
                &linked_ocel,
                None,
                count_options,
            ),
            None => Vec::new(),
        };
        let eventually_follows_constraints = match req.eventually_follows_constraints {
            Some(eventually_follows_options) => {
                auto_discover_eventually_follows(&linked_ocel, None, eventually_follows_options)
            }
            None => Vec::new(),
        };
        AutoDiscoverConstraintsResponse {
            count_constraints: count_constraints
                .into_iter()
                .map(|c| c.constraint)
                .collect(),
            eventually_follows_constraints: eventually_follows_constraints
                .into_iter()
                .map(|efc| efc.constraint)
                .collect(),
        }
    }))
}
