use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

// use plotly::{common::Title, layout::Axis, Histogram, Layout, Plot};

use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};

use crate::{
    binding_box::{
        structs::{
            BindingBoxTreeNode, Constraint, EventVariable, Filter, ObjectVariable, SizeFilter,
        },
        BindingBox, BindingBoxTree,
    },
    discovery::{RNG_SEED, SAMPLE_FRAC, SAMPLE_MIN_NUM_INSTANCES},
    preprocessing::linked_ocel::{EventOrObjectIndex, IndexLinkedOCEL, OCELNodeRef},
};

use super::advanced::EventOrObjectType;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum RefType {
    Object,
    ObjectReversed,
    Event,
    EventReversed,
}
pub fn build_frequencies_from_graph<I: Iterator<Item = EventOrObjectType>>(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    ocel_types: I,
) -> Vec<CountConstraint> {
    let now = Instant::now();
    let mut ret = Vec::new();
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    for t in ocel_types {
        let instances: Option<Vec<_>> = match &t {
            EventOrObjectType::Event(et) => {
                if let Some(o_indices) = ocel.events_of_type.get(et) {
                    let sample_count = if o_indices.len() >= SAMPLE_MIN_NUM_INSTANCES {
                        (o_indices.len() as f32 * SAMPLE_FRAC).ceil() as usize
                    } else {
                        o_indices.len()
                    };
                    let instances = o_indices
                        .iter()
                        .choose_multiple(&mut rng, sample_count)
                        .into_iter()
                        .map(|i| EventOrObjectIndex::Event(*i))
                        .collect();
                    Some(instances)
                } else {
                    None
                }
            }
            EventOrObjectType::Object(ot) => {
                if let Some(o_indices) = ocel.objects_of_type.get(ot) {
                    let sample_count = if o_indices.len() >= SAMPLE_MIN_NUM_INSTANCES {
                        (o_indices.len() as f32 * SAMPLE_FRAC).ceil() as usize
                    } else {
                        o_indices.len()
                    };
                    let instances = o_indices
                        .iter()
                        .choose_multiple(&mut rng, sample_count)
                        .into_iter()
                        .map(|i| EventOrObjectIndex::Object(*i))
                        .collect();
                    Some(instances)
                } else {
                    None
                }
            }
        };
        // println!("Sampled instances: {:?}",instances);
        if let Some(instances) = instances {
            ret.extend(get_count_constraint_for_supporting_instances(
                ocel,
                coverage,
                instances.into_iter(),
                t,
            ));
        }
    }
    // println!("Graph Count Discovery took {:?}", now.elapsed());
    ret
}

pub fn get_count_constraint_for_supporting_instances<I: Iterator<Item = EventOrObjectIndex>>(
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
        if let Some(rels) = ocel.symmetric_rels.get(&o_index) {
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
            let mean = counts.iter().sum::<usize>() as f32 / counts.len() as f32;
            if mean > 0.0 && mean <= 30.0 {
                // Otherwise, not interesting (i.e., no values > 0)
                let std_deviation = counts
                    .iter()
                    .map(|c| {
                        let diff = mean - *c as f32;
                        diff * diff
                    })
                    .sum::<f32>()
                    .sqrt();
                // TODO: Decide if > 0?
                if std_deviation >= 0.0 {
                    // Otherwise, not interesting (no deviation between values)
                    for (min, max) in get_range_with_coverage(counts, coverage, mean, std_deviation)
                    {
                        println!("Counts {} Mean {} stdDev {} for min {} max {} FOR {ref_type:?} {ocel_type}",counts.len(),mean, std_deviation,min,max);
                        ret.push(CountConstraint {
                            min_count: min,
                            max_count: max,
                            root_type: supporting_instance_type.clone(),
                            // root_type: match &ref_type {
                            //     RefType::ObjectReversed => {
                            //         EventOrObjectType::Object(ocel_type.clone())
                            //     }
                            //     RefType::EventReversed => {
                            //         EventOrObjectType::Event(ocel_type.clone())
                            //     }
                            //     _ => supporting_instance_type.clone(),
                            // },
                            // related_type: ocel_type.clone(),
                            related_type: match &ref_type {
                                RefType::Object | RefType::ObjectReversed => EventOrObjectType::Object(ocel_type.clone()),
                                RefType::Event | RefType::EventReversed => EventOrObjectType::Event(ocel_type.clone()),
                            },
                            ocel_relation_flipped: match &ref_type {
                                RefType::ObjectReversed |RefType::EventReversed => true,
                                _ => false
                            }
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
            2 * mean_usize,
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
    while steps < 10_000
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

#[derive(Debug, Clone)]
pub struct CountConstraint {
    pub min_count: usize,
    pub max_count: usize,
    pub root_type: EventOrObjectType,
    // This is the types of items we constraint in their count!
    pub related_type: EventOrObjectType,
    pub ocel_relation_flipped: bool,
}

impl CountConstraint {
    pub fn get_constraint_name(&self) -> String {
        format!(
            "{} - {} {} per {}",
            self.min_count,
            self.max_count,
            self.related_type.inner(),
            self.root_type.inner()
        )
    }
    pub fn get_full_tree(&self) -> BindingBoxTree {
        // let child_name = "A".to_string();
        let inner_child_name = "A".to_string();
        let inner_variable = 0;
        let mut subtree = self.to_subtree(inner_child_name.clone(), inner_variable, 1);
        // match &mut subtree.nodes[0] {
        //     BindingBoxTreeNode::Box(_, child) => {
        //         child[0] = 2;
        //     },
        //     _ => todo!("Should be a Box")
        // }
        // subtree.nodes.insert(
        //     0,
        //     BindingBoxTreeNode::Box(
        //         BindingBox {
        //             new_event_vars: match &self.root_type {
        //                 EventOrObjectType::Event(et) => vec![(
        //                     EventVariable(inner_variable),
        //                     vec![et.clone()].into_iter().collect(),
        //                 )]
        //                 .into_iter()
        //                 .collect(),
        //                 _ => HashMap::default(),
        //             },
        //             new_object_vars: match &self.root_type {
        //                 EventOrObjectType::Object(ot) => vec![(
        //                     ObjectVariable(inner_variable),
        //                     vec![ot.clone()].into_iter().collect(),
        //                 )]
        //                 .into_iter()
        //                 .collect(),
        //                 _ => HashMap::default(),
        //             },
        //             filters: Vec::default(),
        //             size_filters: Vec::default(),
        //             constraints: vec![Constraint::SAT {
        //                 child_names: vec![child_name.clone()],
        //             }],
        //         },
        //         vec![1],
        //     ),
        // );
        // subtree.edge_names.insert((0,1), child_name);
        // subtree.edge_names.insert((1,2), inner_child_name);
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
            _ => todo!("Expected Box"),
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
                        min: Some(self.min_count),
                        max: Some(self.max_count),
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
