use core::f32;
use std::{
    collections::{HashMap, HashSet},
    f64::MAX,
    sync::Mutex,
};

use graph_discovery::build_frequencies_from_graph;
use itertools::Itertools;
use process_mining::OCEL;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    binding_box::{
        structs::{BindingBoxTreeNode, EventVariable, FilterConstraint, ObjectVariable},
        BindingBox, BindingBoxTree,
    },
    preprocessing::{
        linked_ocel::IndexLinkedOCEL,
        preprocess::{link_ocel_info, LinkedOCEL},
    },
};

use self::evaluation::{get_count_constraint_fraction, get_ef_constraint_fraction};

pub mod evaluation;
pub mod graph_discovery;

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
impl From<&EventuallyFollowsConstraints> for BindingBoxTree {
    fn from(val: &EventuallyFollowsConstraints) -> Self {
        let bbox0 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(0),
                    vec![val.from_event_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: vec![(
                    ObjectVariable(0),
                    val.object_types.iter().cloned().collect(),
                )]
                .into_iter()
                .collect(),
                filter_constraint: vec![FilterConstraint::ObjectAssociatedWithEvent(
                    ObjectVariable(0),
                    EventVariable(0),
                    None,
                )],
            },
            vec![1],
        );
        let bbox1 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(1),
                    vec![val.to_event_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::new(),
                filter_constraint: vec![
                    FilterConstraint::ObjectAssociatedWithEvent(
                        ObjectVariable(0),
                        EventVariable(1),
                        None,
                    ),
                    FilterConstraint::TimeBetweenEvents(
                        EventVariable(0),
                        EventVariable(1),
                        (Some(val.min_seconds), Some(val.max_seconds)),
                    ),
                ],
            },
            vec![],
        );
        BindingBoxTree {
            nodes: vec![bbox0, bbox1],
            size_constraints: vec![((0, 1), (Some(1), None))].into_iter().collect(),
        }
    }
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
                                min_seconds: 0.0,
                                max_seconds: MAX,
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

                            constraint.min_seconds = mean_delay_seconds;
                            constraint.max_seconds = mean_delay_seconds;

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
                                constraint.min_seconds = mean_delay_seconds
                                    - std_dev_factor * delay_seconds_std_deviation;
                                constraint.max_seconds = mean_delay_seconds
                                    + std_dev_factor * delay_seconds_std_deviation;
                            }
                            // Min should be >= 0.0
                            constraint.min_seconds = constraint.min_seconds.max(0.0);

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

#[derive(Debug, Clone)]
pub enum EventOrObject {
    Event,
    Object,
}
#[derive(Debug, Clone)]
pub struct SimpleDiscoveredCountConstraints {
    pub min_count: usize,
    pub max_count: usize,
    pub root_type: String,
    pub root_is: EventOrObject,
    // This is the types of items we constraint in their count!
    pub related_types: Vec<String>,
    pub related_types_are: EventOrObject,
}
// We might want to also return a set of "supporting objects" for each discovered constraints
// These are the objects for which the count constraint is satisfied
// This would allows us to build constraints specifically for the _same_ or the _other_ (i.e., set of objects of same type not supported)
// Would be useful for constructing/discovering targeted OR (or XOR) constraints

// Similiarly, it would be nice to have some sort of input object_ids (only those should be considered)

#[derive(Debug, Clone)]
pub struct CountConstraintInfo {
    pub constraint: SimpleDiscoveredCountConstraints,
    pub supporting_object_ids: HashSet<String>,
    pub cover_fraction: f32,
}
impl SimpleDiscoveredCountConstraints {
    fn get_constraint_name(&self) -> String {
        format!(
            "{} - {} {} per {}",
            self.min_count,
            self.max_count,
            self.related_types.join(", "),
            self.root_type
        )
    }
}

impl From<&SimpleDiscoveredCountConstraints> for BindingBoxTree {
    fn from(val: &SimpleDiscoveredCountConstraints) -> Self {
        let new_ob = match val.root_is {
            EventOrObject::Event => vec![],
            EventOrObject::Object => vec![(
                ObjectVariable(0),
                vec![val.root_type.clone()].into_iter().collect(),
            )],
        };
        let new_ev = match val.root_is {
            EventOrObject::Event => vec![(
                EventVariable(0),
                vec![val.root_type.clone()].into_iter().collect(),
            )],
            EventOrObject::Object => vec![],
        };

        let bbox0 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: new_ev.into_iter().collect(),
                new_object_vars: new_ob.into_iter().collect(),
                filter_constraint: vec![],
            },
            vec![1],
        );

        let new_ob1 = match val.related_types_are {
            EventOrObject::Event => vec![],
            EventOrObject::Object => {
                vec![(
                    ObjectVariable(1),
                    val.related_types.clone().into_iter().collect(),
                )]
            }
        };
        let new_ev1 = match val.related_types_are {
            EventOrObject::Event => {
                vec![(
                    EventVariable(1),
                    val.related_types.clone().into_iter().collect(),
                )]
            }
            EventOrObject::Object => vec![],
        };

        let bbox1 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: new_ev1.into_iter().collect(),
                new_object_vars: new_ob1.into_iter().collect(),
                filter_constraint: vec![match val.root_is {
                    // Must be event, as there are no E2E
                    EventOrObject::Event => FilterConstraint::ObjectAssociatedWithEvent(
                        ObjectVariable(1),
                        EventVariable(0),
                        None,
                    ),

                    EventOrObject::Object => match val.related_types_are {
                        EventOrObject::Event => FilterConstraint::ObjectAssociatedWithEvent(
                            ObjectVariable(0),
                            EventVariable(1),
                            None,
                        ),
                        EventOrObject::Object => FilterConstraint::ObjectAssociatedWithObject(
                            ObjectVariable(0),
                            ObjectVariable(1),
                            None,
                        ),
                    },
                }],
            },
            vec![],
        );
        BindingBoxTree {
            nodes: vec![bbox0, bbox1],
            size_constraints: vec![((0, 1), (Some(val.min_count), Some(val.max_count)))]
                .into_iter()
                .collect(),
        }
    }
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
    options: &CountConstraintOptions,
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
            min_count: mean.round() as usize,
            max_count: mean.round() as usize,
            root_type: object_type,
            root_is: EventOrObject::Object,
            related_types: vec![event_type],
            related_types_are: EventOrObject::Event,
        };

        while get_count_constraint_fraction(linked_ocel, &constraint, &rel_object_ids, false).0
            < options.cover_fraction
        {
            std_dev_factor += 0.003;
            constraint.min_count = (mean - std_dev_factor * std_deviation).round() as usize;
            constraint.max_count = (mean + std_dev_factor * std_deviation).round() as usize;
        }

        // For now, do not discover constraints with huge range; Those are most of the time not desired
        if constraint.max_count - constraint.min_count > 25 || constraint.max_count > 100 {
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

#[derive(Debug)]
pub struct AutoDiscoveredORConstraint(
    pub EventuallyFollowsConstraints,
    pub SimpleDiscoveredCountConstraints,
);

impl AutoDiscoveredORConstraint {
    fn get_constraint_name(&self) -> String {
        format!(
            "OR {} / {}",
            self.0.get_constraint_name(),
            self.1.get_constraint_name()
        )
    }
}
impl From<&AutoDiscoveredORConstraint> for BindingBoxTree {
    fn from(val: &AutoDiscoveredORConstraint) -> Self {
        let object_type = val.1.root_type.clone();
        if val.0.object_types.len() > 1 {
            println!("=== Multiple object types: {:?}", val.0.object_types);
        }
        if !val.0.object_types.contains(&object_type) {
            panic!("!!! No object overlap in OR constraint");
        }
        let root_node = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: HashMap::default(),
                new_object_vars: val
                    .0
                    .object_types
                    .iter()
                    .enumerate()
                    .map(|(i, ot)| (ObjectVariable(i), vec![ot.clone()].into_iter().collect()))
                    .collect(),
                filter_constraint: Vec::default(),
            },
            vec![1],
        );
        let or_node = BindingBoxTreeNode::OR(2, 3);
        let count_node = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(0),
                    val.1.related_types.iter().cloned().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::default(),
                filter_constraint: vec![FilterConstraint::ObjectAssociatedWithEvent(
                    ObjectVariable(0),
                    EventVariable(0),
                    None,
                )],
            },
            vec![],
        );
        let ef_node1 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(1),
                    vec![val.0.from_event_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::default(),
                filter_constraint: vec![FilterConstraint::ObjectAssociatedWithEvent(
                    ObjectVariable(0),
                    EventVariable(1),
                    None,
                )],
            },
            vec![4],
        );

        let ef_node2 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(2),
                    vec![val.0.to_event_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::default(),
                filter_constraint: vec![
                    FilterConstraint::ObjectAssociatedWithEvent(
                        ObjectVariable(0),
                        EventVariable(2),
                        None,
                    ),
                    FilterConstraint::TimeBetweenEvents(
                        EventVariable(1),
                        EventVariable(2),
                        (Some(val.0.min_seconds), Some(val.0.max_seconds)),
                    ),
                ],
            },
            vec![],
        );

        BindingBoxTree {
            nodes: vec![root_node, or_node, count_node, ef_node1, ef_node2],
            size_constraints: vec![
                ((1, 2), (Some(val.1.min_count), Some(val.1.max_count))),
                ((3, 4), (Some(1), None)),
            ]
            .into_iter()
            .collect(),
        }
        // Previous code (might be useful for merging arbitrary trees)
        // let (tree1, tree2): (BindingBoxTree, BindingBoxTree) = match val {
        //     AutoDiscoveredORConstraint::EfOrCount(ef, cc) => {
        //         let tree1: BindingBoxTree = ef.into();
        //         let mut tree2: BindingBoxTree = cc.into();
        //         // Remove first node from tree2 & move size constraint to OR -> new first tree2 node
        //         tree2.nodes = tree2.nodes.into_iter().skip(1).collect();
        //         tree2.size_constraints = tree2
        //             .size_constraints
        //             .into_iter()
        //             .map(|orig_c| {
        //                 let mut c = orig_c;
        //                 c.0 = (0 - tree1.nodes.len() - 1, c.0 .1 - 1);
        //                 c
        //             })
        //             .collect();
        //         (tree1, tree2)
        //     }
        //     AutoDiscoveredORConstraint::CountOrEf(cc, ef) => {
        //         let mut tree1: BindingBoxTree = cc.into();
        //         let tree2: BindingBoxTree = ef.into();
        //         // Remove first node from tree1 & move size constraint to OR -> new first tree1 node
        //         tree1.nodes = tree1.nodes.into_iter().skip(1).collect();
        //         tree1.size_constraints = tree1
        //             .size_constraints
        //             .into_iter()
        //             .map(|orig_c| {
        //                 let mut c = orig_c;
        //                 c.0 = (c.0 .0 - 1, c.0 .1 - 1);
        //                 c
        //             })
        //             .collect();
        //         (tree1, tree2)
        //     }
        // };
        // let root_objs = match tree1.nodes.first().unwrap() {
        //     BindingBoxTreeNode::Box(bbox, _) => bbox.new_object_vars.clone(),
        //     BindingBoxTreeNode::OR(_, _) => todo!(),
        //     BindingBoxTreeNode::AND(_, _) => todo!(),
        //     BindingBoxTreeNode::NOT(_) => todo!(),
        // };

        // let mut tree = BindingBoxTree {
        //     nodes: vec![
        //         BindingBoxTreeNode::Box(
        //             BindingBox {
        //                 new_event_vars: vec![].into_iter().collect(),
        //                 new_object_vars: root_objs,
        //                 filter_constraint: vec![],
        //             },
        //             vec![1],
        //         ),
        //         BindingBoxTreeNode::OR(2, 2 + tree1.nodes.len()),
        //     ],
        //     size_constraints: vec![].into_iter().collect(),
        // };
        // let mut prev_nodes = 2;
        // let mut prev_ob_vars = 0;
        // let mut prev_ev_vars = 0;

        // for tr in &[tree1, tree2] {
        //     for node in &tr.nodes {
        //         tree.nodes.push(match node {
        //             BindingBoxTreeNode::Box(bbox, cs) => BindingBoxTreeNode::Box(
        //                 BindingBox {
        //                     new_event_vars: bbox
        //                         .new_event_vars
        //                         .iter()
        //                         .map(|(a, b)| (EventVariable(a.0 + prev_ev_vars), b.clone()))
        //                         .collect(),
        //                     new_object_vars: HashMap::new(),
        //                     // new_object_vars: bbox
        //                     //     .new_object_vars
        //                     //     .iter()
        //                     //     .map(|(a, b)| (ObjectVariable(a.0 + prev_ob_vars), b.clone()))
        //                     //     .collect(),
        //                     filter_constraint: bbox
        //                         .filter_constraint
        //                         .iter()
        //                         .map(|fc| match fc {
        //                             FilterConstraint::ObjectAssociatedWithEvent(ov, ev, q) => {
        //                                 FilterConstraint::ObjectAssociatedWithEvent(
        //                                     ObjectVariable(ov.0 + prev_ob_vars),
        //                                     EventVariable(ev.0 + prev_ev_vars),
        //                                     q.clone(),
        //                                 )
        //                             }
        //                             FilterConstraint::ObjectAssociatedWithObject(ov1, ov2, q) => {
        //                                 FilterConstraint::ObjectAssociatedWithObject(
        //                                     ObjectVariable(ov1.0 + prev_ob_vars),
        //                                     ObjectVariable(ov2.0 + prev_ob_vars),
        //                                     q.clone(),
        //                                 )
        //                             }
        //                             FilterConstraint::TimeBetweenEvents(ev1, ev2, t) => {
        //                                 FilterConstraint::TimeBetweenEvents(
        //                                     EventVariable(ev1.0 + prev_ev_vars),
        //                                     EventVariable(ev2.0 + prev_ev_vars),
        //                                     *t,
        //                                 )
        //                             }
        //                         })
        //                         .collect(),
        //                 },
        //                 cs.iter().map(|i| i + prev_nodes).collect_vec(),
        //             ),
        //             _ => todo!("Other box tree nodes are not supported for merging!"),
        //         })
        //     }
        //     for ((n_index1, n_index2), size_constr) in &tr.size_constraints {
        //         tree.size_constraints
        //             .insert((n_index1 + prev_nodes, n_index2 + prev_nodes), *size_constr);
        //     }
        //     prev_nodes += tr.nodes.len();
        //     prev_ev_vars += tr.get_ev_vars().len();
        //     prev_ob_vars += 0; //tr.get_ob_vars().len();
        // }

        // tree
    }
}

pub fn auto_discover_or_constraints(
    ocel: &OCEL,
    linked_ocel: &LinkedOCEL,
    obj_types_per_ev_type: &HashMap<&String, HashSet<&String>>,
    options: ORConstraintOptions,
) -> Vec<AutoDiscoveredORConstraint> {
    let object_types = options.object_types;
    let res = auto_discover_eventually_follows(
        linked_ocel,
        None,
        EventuallyFollowsConstraintOptions {
            object_types,
            cover_fraction: 0.8,
        },
    );
    let discovered_ors: Mutex<Vec<AutoDiscoveredORConstraint>> = Mutex::new(Vec::new());
    res.par_iter().for_each(|c| {
        if c.cover_fraction < 0.9 {
            // Other objects (i.e., not supporting) of this type
            let other_objects_of_type: HashSet<String> = c
                .constraint
                .object_types
                .iter()
                .flat_map(|ot| linked_ocel.objects_of_type.get(ot).unwrap().iter())
                .filter(|obj| !c.supporting_object_ids.contains(&obj.id))
                .map(|obj| obj.id.clone())
                .collect();
            if other_objects_of_type.is_empty() {
                return;
                // continue;
            }
            let res_inner = auto_discover_count_constraints(
                ocel,
                obj_types_per_ev_type,
                linked_ocel,
                Some(other_objects_of_type),
                &CountConstraintOptions {
                    object_types: c.constraint.object_types.clone(),
                    cover_fraction: 0.8,
                },
            );
            for c2 in &res_inner {
                let (cover_frac_orig, _) = get_count_constraint_fraction(
                    linked_ocel,
                    &c2.constraint,
                    &c.supporting_object_ids,
                    false,
                );

                let cover_diff = c2.cover_fraction - cover_frac_orig;
                if cover_diff > 0.5 {
                    discovered_ors
                        .lock()
                        .unwrap()
                        .push(AutoDiscoveredORConstraint(
                            c.constraint.clone(),
                            c2.constraint.clone(),
                        ));
                    // c
                    // c2
                    // discovered_ors
                    println!("{:?}", c2.constraint);
                    println!(
                        "Cover diff: {} = {} - {}",
                        cover_diff, c2.cover_fraction, cover_frac_orig
                    );
                }
            }
        }
    });
    discovered_ors.into_inner().unwrap()
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
    let linked_ocel = link_ocel_info(&ocel.ocel);
    let obj_types_per_ev_type = get_obj_types_per_ev_type(&linked_ocel);
    let count_constraints = match &options.count_constraints {
        Some(count_options) => auto_discover_count_constraints(
            &ocel.ocel,
            &obj_types_per_ev_type,
            &linked_ocel,
            None,
            count_options,
        ),
        None => Vec::new(),
    };
    let eventually_follows_constraints = match options.eventually_follows_constraints {
        Some(eventually_follows_options) => {
            auto_discover_eventually_follows(&linked_ocel, None, eventually_follows_options)
        }
        None => Vec::new(),
    };
    let or_constraints = match options.or_constraints {
        Some(or_constraint_option) => auto_discover_or_constraints(
            &ocel.ocel,
            &linked_ocel,
            &obj_types_per_ev_type,
            or_constraint_option,
        ),
        None => Vec::new(),
    };
    let mut ret = AutoDiscoverConstraintsResponse {
        constraints: Vec::new(),
    };
    for cc in &count_constraints {
        ret.constraints
            .push((cc.constraint.get_constraint_name(), (&cc.constraint).into()))
    }
    // TODO: Fully integrate
    if let Some(count_opts) = &options.count_constraints {
        for cc in build_frequencies_from_graph(&ocel, count_opts.cover_fraction) {
            ret.constraints
                .push((cc.get_constraint_name(), (&cc).into()))
        }
    }
    for ef in &eventually_follows_constraints {
        ret.constraints
            .push((ef.constraint.get_constraint_name(), (&ef.constraint).into()))
    }

    for or in &or_constraints {
        ret.constraints.push((or.get_constraint_name(), or.into()))
    }

    ret
}
