use std::collections::{HashMap, HashSet};

use process_mining::{
    event_log::ocel::ocel_struct::{OCELEvent, OCELObject, OCELRelationship},
    OCEL,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn get_object_events_map(ocel: &OCEL) -> HashMap<String, Vec<String>> {
    let mut object_events_map: HashMap<String, Vec<String>> = ocel
        .objects
        .iter()
        .map(|o| (o.id.clone(), Vec::new()))
        .collect();

    for e in &ocel.events {
        let rels = get_relationships(e);
        for r in rels {
            object_events_map
                .get_mut(&r.object_id)
                .unwrap()
                .push(e.id.clone());
        }
    }
    object_events_map
}

pub fn get_events_of_type_associated_with_objects<'a>(
    linked_ocel: &'a LinkedOCEL,
    event_type: &String,
    object_ids: Vec<String>,
) -> Vec<&'a OCELEvent> {
    if object_ids.is_empty() {
        return linked_ocel.events_of_type.get(event_type).unwrap().clone();
    }
    let mut sorted_object_ids = object_ids.clone();
    sorted_object_ids.sort_by(|a, b| {
        linked_ocel
            .object_events_map
            .get(a)
            .unwrap()
            .len()
            .cmp(&linked_ocel.object_events_map.get(b).unwrap().len())
    });
    let mut intersection: HashSet<&String> = linked_ocel
        .object_events_map
        .get(&sorted_object_ids[0])
        .unwrap()
        .iter()
        .filter(|ev_id| &linked_ocel.event_map.get(*ev_id).unwrap().event_type == event_type)
        .collect();
    for i in 1..sorted_object_ids.len() {
        let other = &sorted_object_ids[i];
        let other_map: HashSet<&String> = linked_ocel
            .object_events_map
            .get(other)
            .unwrap()
            .iter()
            .collect();
        intersection.retain(|ev| other_map.contains(ev))
    }
    return intersection
        .iter()
        .map(|ev_id| *linked_ocel.event_map.get(*ev_id).unwrap())
        .collect();
}

pub fn get_relationships(ev: &OCELEvent) -> Vec<OCELRelationship> {
    match &ev.relationships {
        Some(rels) => rels.clone(),
        None => Vec::default(),
    }
}

#[derive(Debug, Clone)]
pub struct LinkedOCEL<'a> {
    pub event_map: HashMap<String, &'a OCELEvent>,
    pub object_map: HashMap<String, &'a OCELObject>,
    pub events_of_type: HashMap<String, Vec<&'a OCELEvent>>,
    #[allow(dead_code)]
    pub objects_of_type: HashMap<String, Vec<&'a OCELObject>>,
    pub object_events_map: HashMap<String, Vec<String>>,
}

pub fn link_ocel_info(ocel: &OCEL) -> LinkedOCEL {
    let event_map: HashMap<String, &OCELEvent> = ocel
        .events
        .par_iter()
        .map(|ev| (ev.id.clone(), ev))
        .collect();
    let object_map: HashMap<String, &OCELObject> = ocel
        .objects
        .par_iter()
        .map(|obj| (obj.id.clone(), obj))
        .collect();

    let events_of_type: HashMap<String, Vec<&OCELEvent>> = ocel
        .event_types
        .par_iter()
        .map(|ev_type| {
            (
                ev_type.name.clone(),
                ocel.events
                    .iter()
                    .filter(|ev| ev.event_type == ev_type.name)
                    .collect(),
            )
        })
        .collect();

    let objects_of_type: HashMap<String, Vec<&OCELObject>> = ocel
        .object_types
        .par_iter()
        .map(|obj_type| {
            (
                obj_type.name.clone(),
                ocel.objects
                    .iter()
                    .filter(|ev| ev.object_type == obj_type.name)
                    .collect(),
            )
        })
        .collect();
    let object_events_map = get_object_events_map(ocel);
    LinkedOCEL {
        event_map,
        object_map,
        events_of_type,
        objects_of_type,
        object_events_map,
    }
}
