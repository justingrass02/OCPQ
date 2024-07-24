use std::collections::HashMap;

// use plotly::{common::Title, layout::Axis, Histogram, Layout, Plot};

use crate::{
    discovery::EventOrObject,
    preprocessing::linked_ocel::{IndexLinkedOCEL, OCELNodeRef},
};

use super::SimpleDiscoveredCountConstraints;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum RefType {
    Object,
    ObjectReversed,
    Event,
}
pub fn build_frequencies_from_graph(
    ocel: &IndexLinkedOCEL,
    coverage: f32,
    object_types: &[String],
) -> Vec<SimpleDiscoveredCountConstraints> {
    let mut ret = Vec::new();
    for ot in object_types {
        if let Some(o_indices) = ocel.objects_of_type.get(ot) {
            let mut total_map: HashMap<_, Vec<usize>> = HashMap::new();
            for o_index in o_indices {
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
                if let Some(rels) = ocel.get_symmetric_rels_ob(o_index) {
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
            println!("\n=== {} ===", ot);
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
                        if std_deviation > 0.0 {
                            // Otherwise, not interesting (no deviation between values)
                            let (min, max) =
                                get_range_with_coverage(counts, coverage, mean, std_deviation);
                                if *ref_type != RefType::ObjectReversed {
                                    // TODO: Fix Reversed Objects!
                                    // For that, we need to modify SimpleDiscoveredCountConstraints further
                                    ret.push(SimpleDiscoveredCountConstraints {
                                        min_count: min,
                                        max_count: max, root_type: if *ref_type == RefType::ObjectReversed { ocel_type.clone() } else {ot.clone()} ,
                                        root_is: crate::discovery::EventOrObject::Object,
                                        related_types: vec![if *ref_type == RefType::ObjectReversed {ot.clone()} else  {ocel_type.clone()}],
                                        related_types_are: if *ref_type == RefType::Event { EventOrObject::Event } else { EventOrObject::Object}
                                    });
                                }
                            println!(
                                "{min} - {max} for {ocel_type} ({ref_type:?}): {:.2} mean, {:.2} std-deviation",
                                mean, std_deviation
                            );
                            // plot_histogram(
                            //     counts,
                            //     format!(
                            //         "Number of related {:?} ({:?}) per {:?}",
                            //         ocel_type, ref_type, ot.name
                            //     ),
                            //     format!("./plots/{}-{:?}-{}.svg", ot.name, ref_type, ocel_type),
                            // );
                        }
                    }
                })
        }
    }
    ret
}

fn get_range_with_coverage(
    values: &[usize],
    coverage: f32,
    mean: f32,
    std_deviation: f32,
) -> (usize, usize) {
    let mut min = mean.floor() as usize;
    let mut max = mean.floor() as usize;
    let mut steps = 0;
    let step_size = (0.01 * std_deviation).round().max(1.0) as usize;
    let coverage_min_count = (values.len() as f32 * coverage).ceil() as usize;
    while steps < 10_000 && values.iter().filter(|v| **v >= min && **v <= max).count() < coverage_min_count {
        if min > 0 {
            min -= step_size;
        }
        max += step_size;
        steps += 1;
    }
    (min, max)
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use process_mining::OCEL;

    use crate::preprocessing::linked_ocel::link_ocel_info;

    use super::build_frequencies_from_graph;

    #[test]
    fn test_build_frequencies_from_graph() {
        let bytes = include_bytes!("../../../data/order-management.json");
        let now = Instant::now();
        let ocel: OCEL = serde_json::from_slice(bytes).unwrap();
        println!("Loaded OCEL in {:?}", now.elapsed());
        let now = Instant::now();
        let index_ocel = link_ocel_info(ocel);
        println!("Linked OCEL in {:?}\n", now.elapsed());
        let now = Instant::now();
        build_frequencies_from_graph(
            &index_ocel,
            0.7,
            &index_ocel
                .ocel
                .object_types
                .iter()
                .map(|ot| ot.name.clone())
                .collect::<Vec<_>>(),
        );
        println!("\nBuild Frequencies from Graph in {:?}", now.elapsed());
    }
}
