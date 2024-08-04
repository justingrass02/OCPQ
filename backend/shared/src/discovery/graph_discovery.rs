use core::f64;
use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, HashSet},
    time::Instant,
};

// use plotly::{common::Title, layout::Axis, Histogram, Layout, Plot};

use itertools::Itertools;
use process_mining::ocel::ocel_struct::OCELType;
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};

use crate::{
    binding_box::{
        structs::{
            BindingBoxTreeNode, Constraint, EventVariable, Filter, ObjectVariable, SizeFilter,
            Variable,
        },
        BindingBox, BindingBoxTree,
    },
    discovery::{
        advanced::{
            binding_to_instances, generate_sample_bindings, get_labeled_instances, label_bindings,
        },
        RNG_SEED, SAMPLE_FRAC, SAMPLE_MIN_NUM_INSTANCES,
    },
    preprocessing::linked_ocel::{EventOrObjectIndex, IndexLinkedOCEL, OCELNodeRef, ObjectIndex},
};

use super::advanced::EventOrObjectType;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum RefType {
    Object,
    ObjectReversed,
    Event,
    EventReversed,
}

pub fn get_instances(
    ocel: &IndexLinkedOCEL,
    ocel_type: &EventOrObjectType,
) -> Vec<EventOrObjectIndex> {
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let instances: Option<Vec<_>> = match &ocel_type {
        EventOrObjectType::Event(et) => {
            if let Some(o_indices) = ocel.events_of_type.get(et) {
                let instances = if o_indices.len() >= SAMPLE_MIN_NUM_INSTANCES {
                    let sample_count = (o_indices.len() as f32 * SAMPLE_FRAC).ceil() as usize;
                    o_indices.iter().choose_multiple(&mut rng, sample_count)
                } else {
                    o_indices.iter().collect()
                };
                Some(
                    instances
                        .into_iter()
                        .map(|i| EventOrObjectIndex::Event(*i))
                        .collect(),
                )
            } else {
                None
            }
        }
        EventOrObjectType::Object(ot) => {
            if let Some(o_indices) = ocel.objects_of_type.get(ot) {
                let instances = if o_indices.len() >= SAMPLE_MIN_NUM_INSTANCES {
                    let sample_count = (o_indices.len() as f32 * SAMPLE_FRAC).ceil() as usize;
                    o_indices.iter().choose_multiple(&mut rng, sample_count)
                } else {
                    o_indices.iter().collect()
                };
                Some(
                    instances
                        .into_iter()
                        .map(|i| EventOrObjectIndex::Object(*i))
                        .collect(),
                )
            } else {
                None
            }
        }
    };
    instances.unwrap_or_default()
}

pub fn discover_count_constraints(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    ocel_type: EventOrObjectType,
) -> Vec<CountConstraint> {
    let now = Instant::now();
    let mut ret = Vec::new();
    let instances: Vec<_> = get_instances(ocel, &ocel_type);
    ret.extend(discover_count_constraints_for_supporting_instances(
        ocel,
        coverage,
        instances.into_iter(),
        ocel_type,
    ));
    println!("Graph Count Discovery took {:?}", now.elapsed());
    ret
}

pub fn discover_count_constraints_for_supporting_instances<
    It: Borrow<EventOrObjectIndex>,
    I: Iterator<Item = It>,
>(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    supporting_instances: I,
    supporting_instance_type: EventOrObjectType,
) -> Vec<CountConstraint> {
    let mut ret = Vec::new();
    let mut total_map: HashMap<_, Vec<usize>> = HashMap::new();
    for o_index in supporting_instances {
        let mut map: HashMap<_, usize> = ocel
            .ocel
            .event_types
            .iter()
            .map(|et| ((RefType::Event, et.name.clone()), 0))
            .chain(ocel.ocel.object_types.iter().flat_map(|ot| {
                vec![
                    ((RefType::Object, ot.name.clone()), 0),
                    ((RefType::ObjectReversed, ot.name.clone()), 0),
                ]
            }))
            .collect();
        if let Some(rels) = ocel.symmetric_rels.get(o_index.borrow()) {
            for (index, reversed, _qualifier) in rels {
                let (ref_type, ocel_type) = match ocel.ob_or_ev_by_index(*index).unwrap() {
                    OCELNodeRef::Event(e) => (RefType::Event, e.event_type.clone()),
                    OCELNodeRef::Object(o) => (
                        if *reversed {
                            RefType::ObjectReversed
                        } else {
                            RefType::Object
                        },
                        o.object_type.clone(),
                    ),
                };
                *map.entry((ref_type, ocel_type)).or_default() += 1;
            }
        }
        for (k, val) in map {
            total_map.entry(k).or_default().push(val);
        }
    }
    // println!("\n=== {} ===", ot);
    total_map
        .iter()
        .for_each(|((ref_type, ocel_type), counts)| {
            let n = counts.len() as f32;
            let mean = counts.iter().sum::<usize>() as f32 / n;
            let min = counts.iter().min().unwrap_or(&0);
            let max = counts.iter().max().unwrap_or(&usize::MAX);
            
            if mean > 0.0 && mean <= 30.0 {
                // Otherwise, not interesting (i.e., no values > 0)
                let std_deviation = (counts
                    .iter()
                    .map(|c| {
                        let diff = mean - *c as f32;
                        diff * diff
                    })
                    .sum::<f32>()
                    / n)
                    .sqrt();
                // TODO: Decide if > 0?
                if std_deviation >= 0.0 {
                    // Otherwise, not interesting (no deviation between values)
                    for (c_min, c_max) in get_range_with_coverage(counts, coverage, mean, std_deviation)
                    {
                        // println!("Counts {} Mean {} stdDev {} for min {} max {} FOR {ref_type:?} {ocel_type}",counts.len(),mean, std_deviation,min,max);
                        ret.push(CountConstraint {
                            min_count: if c_min > 0 && c_min < *min { None } else { Some(c_min) },
                            max_count: if c_max > *max { None } else { Some(c_max)},
                            root_type: supporting_instance_type.clone(),
                            related_type: match &ref_type {
                                RefType::Object | RefType::ObjectReversed => {
                                    EventOrObjectType::Object(ocel_type.clone())
                                }
                                RefType::Event | RefType::EventReversed => {
                                    EventOrObjectType::Event(ocel_type.clone())
                                }
                            },
                            ocel_relation_flipped: match &ref_type {
                                RefType::ObjectReversed | RefType::EventReversed => true,
                                _ => false,
                            },
                        });
                    }
                }
            }
        });
    ret
}

fn get_range_with_coverage(
    values: &[usize],
    coverage: f32,
    mean: f32,
    std_deviation: f32,
) -> HashSet<(usize, usize)> {
    let mean_usize = mean.floor() as usize;
    let step_size = (0.01 * std_deviation).round().max(1.0) as usize;
    let mut ret: HashSet<_> = vec![
        get_range_with_coverage_inner(
            values,
            coverage,
            mean_usize,
            step_size,
            Direction::Symmetric,
        ),
        get_range_with_coverage_inner(values, coverage, 0, step_size, Direction::Increase),
        get_range_with_coverage_inner(
            values,
            coverage,
            10 + 10 * mean_usize,
            step_size,
            Direction::Decrease,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut remove_from_ret = HashSet::new();
    for (min, max) in &ret {
        if ret
            .iter()
            .any(|(min2, max2)| min2 >= min && max2 <= max && (min, max) != (min2, max2))
        {
            remove_from_ret.insert((*min, *max));
        }
    }
    for k in remove_from_ret {
        ret.remove(&k);
    }
    ret
}

#[derive(PartialEq)]
enum Direction {
    Decrease,
    Increase,
    Symmetric,
}

fn get_range_with_coverage_inner(
    values: &[usize],
    coverage: f32,
    start: usize,
    step_size: usize,
    direction: Direction,
) -> Vec<(usize, usize)> {
    let mut min = start;
    let mut max = start;
    let mut steps = 0;
    let coverage_min_count = (values.len() as f32 * coverage).ceil() as usize;
    while steps < 1000
        && values.iter().filter(|v| **v >= min && **v <= max).count() < coverage_min_count
    {
        // Watch out for overflows!
        if direction != Direction::Increase {
            if min >= step_size {
                min -= step_size;
            } else {
                min = 0;
            }
        }

        if direction != Direction::Decrease {
            max += step_size;
        }
        steps += 1;
    }
    if steps >= 1000 {
        println!("[Warning!] Could not find coverage range after 1000 steps.");
        return Vec::default();
    }
    vec![(min, max)]
}

fn get_seconds_range_with_coverage(
    values: &[Option<f64>],
    coverage: f32,
    start: f64,
    step_size: f64,
    direction: Direction,
) -> Option<(f64, f64)> {
    let mut min = start;
    let mut max = start;
    let mut steps = 0;
    let coverage_min_count = (values.len() as f32 * coverage).ceil() as usize;
    while steps < 1000
        && values
            .iter()
            .filter(|v| v.is_some_and(|v| v >= min && v <= max))
            .count()
            < coverage_min_count
    {
        // Watch out for overflows!
        if direction != Direction::Increase {
            if min >= step_size {
                min -= step_size;
            } else {
                min = 0.0;
            }
        }

        if direction != Direction::Decrease {
            max += step_size;
        }
        steps += 1;
    }
    if steps >= 1000 {
        println!("[Warning!] Could not find coverage range after 1000 steps.");
        return None;
    }
    Some((min, max))
}

// fn plot_histogram<S: AsRef<str>, P: AsRef<std::path::Path>>(
//     counts: &Vec<usize>,
//     title: S,
//     filename: P,
// ) {
//     let mut plot = Plot::new();
//     let trace = Histogram::new(counts.clone());
//     plot.add_trace(trace);
//     let layout = Layout::new()
//         .title(Title::new(title.as_ref()))
//         .x_axis(Axis::new().dtick(1.0).title(Title::new("Value")))
//         .y_axis(Axis::new().title(Title::new("Count")))
//         .bar_gap(0.05)
//         .bar_group_gap(0.05);
//     plot.set_layout(layout);
//     plot.write_image(filename, plotly::ImageFormat::SVG, 800, 600, 1.0)
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountConstraint {
    pub min_count: Option<usize>,
    pub max_count: Option<usize>,
    pub root_type: EventOrObjectType,
    // This is the types of items we constraint in their count!
    pub related_type: EventOrObjectType,
    pub ocel_relation_flipped: bool,
}

impl CountConstraint {
    pub fn get_constraint_name(&self) -> String {
        format!(
            "{} - {} '{}' per '{}'",
            if let Some(min) = self.min_count {min} else { 0 }, 
            if let Some(max) = self.max_count {max.to_string()} else { "∞".to_string() }, 
            self.related_type.inner(),
            self.root_type.inner()
        )
    }
    pub fn get_full_tree(&self) -> BindingBoxTree {
        // let child_name = "A".to_string();
        let inner_child_name = "A".to_string();
        let inner_variable = 0;
        let mut subtree = self.to_subtree(inner_child_name.clone(), inner_variable, 1);
        match &mut subtree.nodes[0] {
            BindingBoxTreeNode::Box(bbox, _) => {
                match &self.root_type {
                    EventOrObjectType::Event(et) => bbox.new_event_vars.insert(
                        EventVariable(inner_variable),
                        vec![et.clone()].into_iter().collect(),
                    ),
                    EventOrObjectType::Object(ot) => bbox.new_object_vars.insert(
                        ObjectVariable(inner_variable),
                        vec![ot.clone()].into_iter().collect(),
                    ),
                };
            }
            _ => todo!("Expected a BindingBox"),
        }
        subtree
    }
    pub fn to_subtree(
        &self,
        child_name: String,
        inner_variable: usize,
        new_variable: usize,
    ) -> BindingBoxTree {
        let bbox0 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: HashMap::default(),
                new_object_vars: HashMap::default(),
                filters: vec![],
                size_filters: vec![],
                constraints: vec![Constraint::SizeFilter {
                    filter: SizeFilter::NumChilds {
                        child_name: child_name.clone(),
                        min: self.min_count,
                        max: self.max_count,
                    },
                }],
            },
            vec![1],
        );

        let new_ob1 = match &self.related_type {
            EventOrObjectType::Object(_t) => {
                vec![(
                    ObjectVariable(new_variable),
                    vec![self.related_type.inner().to_string()]
                        .into_iter()
                        .collect(),
                )]
            }
            _ => Vec::default(),
        };
        let new_ev1 = match &self.related_type {
            EventOrObjectType::Event(_t) => {
                vec![(
                    EventVariable(new_variable),
                    vec![self.related_type.inner().to_string()]
                        .into_iter()
                        .collect(),
                )]
            }
            _ => Vec::default(),
        };

        let bbox1 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: new_ev1.into_iter().collect(),
                new_object_vars: new_ob1.into_iter().collect(),
                filters: vec![match self.root_type {
                    // Must be object, as there are no E2E
                    EventOrObjectType::Event(_) => Filter::O2E {
                        object: ObjectVariable(new_variable),
                        event: EventVariable(inner_variable),
                        qualifier: None,
                    },

                    EventOrObjectType::Object(_) => match self.related_type {
                        EventOrObjectType::Event(_) => Filter::O2E {
                            object: ObjectVariable(inner_variable),
                            event: EventVariable(new_variable),
                            qualifier: None,
                        },
                        EventOrObjectType::Object(_) => Filter::O2O {
                            object: if !self.ocel_relation_flipped {
                                ObjectVariable(inner_variable)
                            } else {
                                ObjectVariable(new_variable)
                            },
                            other_object: if !self.ocel_relation_flipped {
                                ObjectVariable(new_variable)
                            } else {
                                ObjectVariable(inner_variable)
                            },
                            qualifier: None,
                        },
                    },
                }],
                size_filters: vec![],
                constraints: vec![],
            },
            vec![],
        );
        BindingBoxTree {
            nodes: vec![bbox0, bbox1],
            edge_names: vec![((0, 1), child_name)].into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EFConstraint {
    pub from_ev_type: String,
    pub to_ev_type: String,
    pub min_duration_sec: Option<f64>,
    pub max_duration_sec: Option<f64>,
    pub for_object_type: String,
}

pub fn discover_ef_constraints(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    object_type: &String,
) -> Vec<EFConstraint> {
    let now = Instant::now();
    let mut ret = Vec::new();
    let instances: Vec<_> = get_instances(ocel, &EventOrObjectType::Object(object_type.clone()));
    ret.extend(discover_ef_constraints_for_supporting_instances(
        ocel,
        coverage,
        instances.into_iter().flat_map(|i| match i {
            EventOrObjectIndex::Object(oi) => Some(oi),
            _ => None,
        }),
        &object_type,
    ));

    println!("Graph Count Discovery took {:?}", now.elapsed());
    ret
}

pub fn discover_ef_constraints_for_supporting_instances<I: Iterator<Item = ObjectIndex>>(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    supporting_instances: I,
    supporting_object_type: &String,
) -> Vec<EFConstraint> {
    let mut ret = Vec::new();
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let mut total_map: HashMap<(&String, &String), Vec<Option<f64>>> = HashMap::new();
    for o_index in supporting_instances {
        if let Some(rels) = ocel.get_symmetric_rels_ob(&o_index) {
            let evs = rels
                .into_iter()
                .flat_map(|(o_or_e_index, _reverse, _qualifier)| match o_or_e_index {
                    EventOrObjectIndex::Event(ei) => ocel.ev_by_index(ei),
                    EventOrObjectIndex::Object(_) => None,
                })
                .collect_vec();
            let evs_num = evs.len();
            let evs = if evs_num >= 100 {
                evs.into_iter()
                    .choose_multiple(&mut rng, (SAMPLE_FRAC * evs_num as f32).ceil() as usize)
            } else {
                evs
            };
            for i in 0..evs.len() {
                let mut min_delay_to: HashMap<&String, Option<f64>> = ocel
                    .ocel
                    .event_types
                    .iter()
                    .map(|t| (&t.name, None))
                    .collect();
                for j in 0..evs.len() {
                    if i != j && evs[i].time <= evs[j].time {
                        let time_diff =
                            (evs[j].time - evs[i].time).num_milliseconds() as f64 / 1000.0;

                        let v = min_delay_to.entry(&evs[j].event_type).or_default();
                        if v.is_none() || v.unwrap() > time_diff {
                            *v = Some(time_diff);
                        }
                    }
                }
                for (to_ev_type, min_delay) in &min_delay_to {
                    total_map
                        .entry((&evs[i].event_type, *to_ev_type))
                        .or_default()
                        .push(*min_delay);
                }
            }
        }
    }
    total_map
        .iter()
        .for_each(|((from_ev_type, to_ev_type), seconds)| {
            let num_ef_with_any_delay = seconds.iter().filter(|s| s.is_some()).count() as f32;
            let fraction_ef_with_any_delay = num_ef_with_any_delay / seconds.len() as f32;
            // println!("EF Fraction: {fraction_ef_with_any_delay} of {coverage}");
            if fraction_ef_with_any_delay >= coverage {
                let n = seconds.iter().flatten().count() as f64;
                let mean = seconds.iter().flatten().sum::<f64>() / n;
                // Otherwise, not interesting (i.e., no values > 0)
                let std_deviation = (seconds
                    .iter()
                    .flatten()
                    .map(|c| {
                        let diff = mean - *c as f64;
                        diff * diff
                    })
                    .sum::<f64>()
                    / n)
                    .sqrt();
                // TODO: Decide if > 0?
                if std_deviation >= 0.0 {
                    // Otherwise, not interesting (no deviation between values)
                    if let Some((min, max)) = get_seconds_range_with_coverage(
                        seconds,
                        coverage,
                        0.0,
                        std_deviation,
                        Direction::Increase,
                    ) {
                        ret.push(EFConstraint {
                            from_ev_type: (*from_ev_type).clone(),
                            to_ev_type: (*to_ev_type).clone(),
                            min_duration_sec: Some(min),
                            max_duration_sec: Some(max),
                            for_object_type: supporting_object_type.clone(),
                        });
                    }
                }
            }
        });
    ret
}

impl EFConstraint {
    pub fn get_constraint_name(&self) -> String {
        format!(
            "Quick '{}' -> '{}' for '{}'",
            self.from_ev_type, self.to_ev_type, self.for_object_type,
        )
    }
    pub fn get_full_tree(&self) -> BindingBoxTree {
        // let child_name = "A".to_string();
        let inner_child_name = "A".to_string();
        let inner_variable = 0;
        let mut subtree = self.to_subtree(inner_child_name.clone(), inner_variable, 1, 2);
        match &mut subtree.nodes[0] {
            BindingBoxTreeNode::Box(bbox, _) => {
                bbox.new_object_vars.insert(
                    ObjectVariable(inner_variable),
                    vec![self.for_object_type.clone()].into_iter().collect(),
                );
            }
            _ => todo!("Expected a BindingBox"),
        }
        subtree
    }
    pub fn to_subtree(
        &self,
        child_name: String,
        inner_variable: usize,
        new_from_ev_var: usize,
        new_to_ev_var: usize,
    ) -> BindingBoxTree {
        let bbox0 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(new_from_ev_var),
                    vec![self.from_ev_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::default(),
                filters: vec![Filter::O2E {
                    object: ObjectVariable(inner_variable),
                    event: EventVariable(new_from_ev_var),
                    qualifier: None,
                }],
                size_filters: vec![],
                constraints: vec![Constraint::SizeFilter {
                    filter: SizeFilter::NumChilds {
                        child_name: child_name.clone(),
                        min: Some(1),
                        max: None,
                    },
                }],
            },
            vec![1],
        );

        let bbox1 = BindingBoxTreeNode::Box(
            BindingBox {
                new_event_vars: vec![(
                    EventVariable(new_to_ev_var),
                    vec![self.to_ev_type.clone()].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                new_object_vars: HashMap::default(),
                filters: vec![
                    Filter::O2E {
                        object: ObjectVariable(inner_variable),
                        event: EventVariable(new_to_ev_var),
                        qualifier: None,
                    },
                    Filter::TimeBetweenEvents {
                        from_event: EventVariable(new_from_ev_var),
                        to_event: EventVariable(new_to_ev_var),
                        min_seconds: self.min_duration_sec,
                        max_seconds: self.max_duration_sec,
                    },
                ],
                size_filters: vec![],
                constraints: vec![],
            },
            vec![],
        );
        BindingBoxTree {
            nodes: vec![bbox0, bbox1],
            edge_names: vec![((0, 1), child_name)].into_iter().collect(),
        }
    }
}

pub fn discover_or_constraints_new(
    ocel: &IndexLinkedOCEL,
    object_type: &String,
) -> Vec<(String,BindingBoxTree)> {
    let now = Instant::now();
    let ocel_type: EventOrObjectType = EventOrObjectType::Object(object_type.clone());
    let mut ret = Vec::new();
    let instances: Vec<_> = get_instances(ocel, &ocel_type);
    let mut count_constraints: HashSet<CountConstraint> = discover_count_constraints_for_supporting_instances(
        ocel,
        0.1,
        instances.iter(),
        ocel_type.clone(),
    ).into_iter().collect();
    count_constraints.extend(discover_count_constraints_for_supporting_instances(
        ocel,
        0.5,
        instances.iter(),
        ocel_type.clone(),
    ));
    count_constraints.extend(discover_count_constraints_for_supporting_instances(
        ocel,
        0.8,
        instances.iter(),
        ocel_type.clone(),
    ));
    let variable = Variable::Object(ObjectVariable(0));
    let bindings = generate_sample_bindings(ocel, &vec![ocel_type.clone()], variable.clone());
    let instances = binding_to_instances(&bindings, variable.clone());
    for cc in &count_constraints {
        // if cc.related_type.inner() != "payment reminder" || cc.min_count < 1 {
        //     continue;
        // }
        // println!("===\n{cc:#?}\n===");
        // println!("Count constraint exists!");
        let cc_subtree = cc.to_subtree("X".to_string(), variable.to_inner(), 1);
        let cc_labeled_bindings = label_bindings(ocel, &bindings, &cc_subtree);
        let cc_sat_count = cc_labeled_bindings.iter().filter(|x| **x).count();
        let ef_constraints: Vec<EFConstraint> = discover_ef_constraints_for_supporting_instances(
            ocel,
            0.85,
            cc_labeled_bindings
                .iter()
                .zip(instances.iter())
                .filter(|(satisfied, _instance)| !**satisfied)
                .flat_map(|(_satisfied, instance)| instance)
                .flat_map(|i| match i {
                    EventOrObjectIndex::Object(oi) => Some(oi),
                    EventOrObjectIndex::Event(_) => None,
                })
                .cloned(),
            object_type,
        );
        for ef_c in ef_constraints {
            // Check if cc OR ef_c is a good candidate
            // for that, first get labeled results for ef_c
            let ef_c_subtree = ef_c.to_subtree("X".to_string(), variable.to_inner(), 2, 3);
            let ef_c_labeled_bindings = label_bindings(ocel, &bindings, &ef_c_subtree);
            let or_sat_count = ef_c_labeled_bindings
                .iter()
                .zip(cc_labeled_bindings.iter())
                .filter(|(ef_sat, cc_sat)| **ef_sat || **cc_sat)
                .count();
            let ef_c_sat_count = ef_c_labeled_bindings.iter().filter(|x| **x).count();
            let good_or_frac =
                or_sat_count as f32 / usize::max(cc_sat_count, ef_c_sat_count) as f32;
            let good_sat_frac = or_sat_count as f32 / bindings.len() as f32;
            // println!("Evaluated OR candidate with scores {good_or_frac} {good_sat_frac} for counts {or_sat_count}");
            if good_sat_frac >= 0.8 && good_or_frac >= 1.15 {
                let or_tree = merge_or_tree(
                    ef_c_subtree,
                    cc_subtree.clone(),
                    ocel_type.clone(),
                    Variable::Object(ObjectVariable(0)),
                );
                // println!(
                //     "Found EF Constraint: {:?} for count constraint {:?}",
                //     ef_c, cc
                // );
                ret.push((format!("Quick '{}' after '{}' OR {}",ef_c.to_ev_type,ef_c.from_ev_type,cc.get_constraint_name()),or_tree));
            }
        }
    }
    println!("OR Graph Discovery took {:?}", now.elapsed());
    ret
}

pub fn merge_or_tree(
    tree1: BindingBoxTree,
    tree2: BindingBoxTree,
    ocel_type: EventOrObjectType,
    input_variable: Variable,
) -> BindingBoxTree {
    let name1 = "A".to_string();
    let name2: String = "B".to_string();
    let mut bbox = BindingBox {
        new_event_vars: HashMap::new(),
        new_object_vars: HashMap::new(),
        filters: Vec::default(),
        size_filters: Vec::default(),
        constraints: vec![Constraint::OR {
            child_names: vec![name1.clone(), name2.clone()],
        }],
    };
    match ocel_type {
        EventOrObjectType::Event(et) => bbox.new_event_vars.insert(
            EventVariable(input_variable.to_inner()),
            vec![et.clone()].into_iter().collect(),
        ),
        EventOrObjectType::Object(ot) => bbox.new_object_vars.insert(
            ObjectVariable(input_variable.to_inner()),
            vec![ot.clone()].into_iter().collect(),
        ),
    };
    let or_box = BindingBoxTreeNode::Box(bbox, vec![1, 1 + tree1.nodes.len()]);
    let mut or_tree = BindingBoxTree {
        nodes: vec![or_box],
        edge_names: HashMap::default(),
    };
    for tn in &tree1.nodes {
        match tn {
            BindingBoxTreeNode::Box(tn_box, tn_children) => {
                or_tree.nodes.push(BindingBoxTreeNode::Box(
                    tn_box.clone(),
                    tn_children.iter().map(|c| c + 1).collect(),
                ))
            }
            _ => {}
        }
    }
    for tn in &tree2.nodes {
        match tn {
            BindingBoxTreeNode::Box(tn_box, tn_children) => {
                or_tree.nodes.push(BindingBoxTreeNode::Box(
                    tn_box.clone(),
                    tn_children
                        .iter()
                        .map(|c| c + 1 + tree1.nodes.len())
                        .collect(),
                ))
            }
            _ => {}
        }
    }
    or_tree.edge_names.insert((0, 1), name1.clone());
    or_tree
        .edge_names
        .insert((0, 1 + tree1.nodes.len()), name2.clone());
    or_tree.edge_names.extend(
        tree1
            .edge_names
            .iter()
            .map(|((from, to), name)| ((*from + 1, to + 1), name.clone())),
    );
    or_tree
        .edge_names
        .extend(tree2.edge_names.iter().map(|((from, to), name)| {
            (
                (*from + 1 + tree1.nodes.len(), to + 1 + tree1.nodes.len()),
                name.clone(),
            )
        }));
    or_tree
}
