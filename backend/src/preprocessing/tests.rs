#[cfg(test)]
#[test]
pub fn test() {
    use std::time::Instant;

    use crate::{
        discovery::{auto_discover_count_constraints, CountConstraintOptions},
        load_ocel::{load_ocel_file, DEFAULT_OCEL_FILE},
    };

    let _now = Instant::now();
    let ocel = load_ocel_file(DEFAULT_OCEL_FILE).unwrap();
    let res = auto_discover_count_constraints(
        &ocel,
        CountConstraintOptions {
            cover_fraction: 0.9,
        },
    );
    println!("{:#?}", res);
    // println!(
    //     "Loaded OCEL with {} events and {} objects in {:?}",
    //     ocel.events.len(),
    //     ocel.objects.len(),
    //     now.elapsed()
    // );
    // // let ocel_relations : HashMap<String,Vec<OCELRelationship>> = ocel.objects.iter().map(|obj| (obj.id.clone(),get_object_relationships(obj))).collect();
    // let linked_ocel = link_ocel_info(&ocel);
    // println!("{:#?}", linked_ocel.object_rels_per_type);
    // // println!("First object: {:?}",o);
}
