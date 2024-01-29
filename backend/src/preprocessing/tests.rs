#[cfg(test)]
#[test]
pub fn test() {
    use std::{
        collections::{HashMap, HashSet},
        time::Instant,
    };

    use process_mining::event_log::ocel::ocel_struct::OCELRelationship;

    use crate::{
        load_ocel::{load_ocel_file, DEFAULT_OCEL_FILE},
        preprocessing::preprocess::{get_object_relationships, link_ocel_info},
    };

    let mut now = Instant::now();
    let ocel = load_ocel_file("ContainerLogistics").unwrap();
    println!(
        "Loaded OCEL with {} events and {} objects in {:?}",
        ocel.events.len(),
        ocel.objects.len(),
        now.elapsed()
    );
    // let ocel_relations : HashMap<String,Vec<OCELRelationship>> = ocel.objects.iter().map(|obj| (obj.id.clone(),get_object_relationships(obj))).collect();
    let linked_ocel = link_ocel_info(&ocel);
    println!("{:#?}", linked_ocel.object_rels_per_type);
    // println!("First object: {:?}",o);
}
