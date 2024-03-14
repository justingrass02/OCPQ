use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    constraints::{CountConstraint, EventType},
    discovery::{EventuallyFollowsConstraints, SimpleDiscoveredCountConstraints},
};

use super::preprocess::{get_events_of_type_associated_with_objects, LinkedOCEL};

#[cfg(test)]
#[test]
pub fn test() {
    use std::{
        collections::{HashMap, HashSet},
        time::Instant,
    };

    use itertools::Itertools;

    use crate::{
        constraints::EventType,
        discovery::{
            auto_discover_count_constraints, auto_discover_eventually_follows,
            evaluation::get_count_constraint_fraction, CountConstraintOptions,
            EventuallyFollowsConstraintOptions,
        },
        load_ocel::{load_ocel_file, DEFAULT_OCEL_FILE},
        preprocessing::preprocess::{get_events_of_type_associated_with_objects, link_ocel_info},
    };

    let _now = Instant::now();
    let ocel = load_ocel_file(DEFAULT_OCEL_FILE).unwrap();
    let linked_ocel = link_ocel_info(&ocel);
    // let res = auto_discover_eventually_follows(
    //     &linked_ocel,
    //     EventuallyFollowsConstraintOptions {
    //         object_types: vec!["orders".to_string()],
    //         cover_fraction: 0.9,
    //     },
    // );
    // let obj_subset = ocel.objects.iter().filter(|obj| obj.object_type == "orders".to_string()).take(10).map(|obj| obj.id.clone()).collect();
    // let res = auto_discover_count_constraints(
    //     &ocel,
    //     &linked_ocel,
    //     // Some(obj_subset),
    //     None,
    //     CountConstraintOptions {
    //         object_types: ocel.object_types.iter().map(|ot| ot.name.clone()).collect(),
    //         cover_fraction: 0.1,
    //     },
    // );

    let res = auto_discover_eventually_follows(
        &linked_ocel,
        None,
        EventuallyFollowsConstraintOptions {
            object_types: ocel.object_types.iter().map(|ot| ot.name.clone()).collect(),
            cover_fraction: 0.8,
        },
    );

    for c in &res {
        if !(&c.constraint.from_event_type == "place order"
            && &c.constraint.to_event_type == "pay order")
        {
            continue;
        }
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
                continue;
            }
            let res_inner = auto_discover_count_constraints(
                &ocel,
                &linked_ocel,
                Some(other_objects_of_type),
                CountConstraintOptions {
                    object_types: c.constraint.object_types.clone(),
                    cover_fraction: 0.8,
                },
            );

            println!("\n\n\n=={:?} ({})==", c.constraint, c.cover_fraction);
            for c2 in &res_inner {
                let (cover_frac_orig, _) = get_count_constraint_fraction(
                    &linked_ocel,
                    &c2.constraint,
                    &c.supporting_object_ids,
                    false
                );

                let cover_diff = c2.cover_fraction - cover_frac_orig;
                if cover_diff > 0.1 {
                    println!("{:?}", c2.constraint);
                    println!(
                        "Cover diff: {} = {} - {}",
                        cover_diff, c2.cover_fraction, cover_frac_orig
                    );
                }
            }
        }
    }
}
