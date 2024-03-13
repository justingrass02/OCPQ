use std::collections::{HashMap, HashSet};

use process_mining::{
    event_log::ocel::ocel_struct::{OCELEvent, OCELObject, OCELRelationship},
    OCEL,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{constraints::EventType, ocel_qualifiers::qualifiers::QualifierAndObjectType};

pub fn get_object_events_map(ocel: &OCEL) -> HashMap<String, Vec<String>> {
    let mut object_events_map: HashMap<String, Vec<String>> = ocel
        .objects
        .iter()
        .map(|o| (o.id.clone(), Vec::new()))
        .collect();

    for e in &ocel.events {
        let rels = get_event_relationships(e);
        for r in rels {
            match object_events_map.get_mut(&r.object_id) {
                Some(o_evs) => o_evs.push(e.id.clone()),
                None => {
                    eprintln!("Malformed OCEL: Event {} relates to object ID {}, which does not belong to any object.",e.id,r.object_id)
                }
            }
        }
    }
    object_events_map
}

pub fn get_events_of_type_associated_with_objects<'a>(
    linked_ocel: &'a LinkedOCEL,
    event_type: &EventType,
    object_ids: Vec<String>,
) -> Vec<&'a OCELEvent> {
    if object_ids.is_empty() {
        return match event_type {
            EventType::Any => linked_ocel.event_map.values().copied().collect(),
            EventType::Exactly { value } => linked_ocel.events_of_type.get(value).unwrap().clone(),
            EventType::AnyOf { values } => linked_ocel
                .event_map
                .values()
                .filter(|e| values.contains(&e.event_type))
                .copied()
                .collect(),
            EventType::AnyExcept { values } => linked_ocel
                .event_map
                .values()
                .filter(|e| !values.contains(&e.event_type))
                .copied()
                .collect(),
        };
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
        .filter(|ev_id| match event_type {
            EventType::Any => true,
            EventType::Exactly { value } => {
                &linked_ocel.event_map.get(*ev_id).unwrap().event_type == value
            }
            EventType::AnyOf { values } => {
                values.contains(&linked_ocel.event_map.get(*ev_id).unwrap().event_type)
            }
            EventType::AnyExcept { values } => {
                !values.contains(&linked_ocel.event_map.get(*ev_id).unwrap().event_type)
            }
        })
        .collect();
    for other in sorted_object_ids.iter().skip(1) {
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

pub fn get_event_relationships(ev: &OCELEvent) -> Vec<OCELRelationship> {
    match &ev.relationships {
        Some(rels) => rels.clone(),
        None => Vec::default(),
    }
}

pub fn get_object_relationships(obj: &OCELObject) -> Vec<OCELRelationship> {
    match &obj.relationships {
        Some(rels) => rels.clone(),
        None => Vec::new(),
    }
}

///
/// Computes [HashMap] linking an object type to the [HashSet] of [QualifierAndObjectType] that objects of that type are linked to (through O2O Relationships)
pub fn get_object_rels_per_type(
    ocel: &OCEL,
    object_map: &HashMap<String, &OCELObject>,
) -> HashMap<String, HashSet<QualifierAndObjectType>> {
    let mut object_to_object_rels_per_type: HashMap<String, HashSet<QualifierAndObjectType>> = ocel
        .object_types
        .iter()
        .map(|t| (t.name.clone(), HashSet::new()))
        .collect();
    for o in &ocel.objects {
        let rels_for_type = object_to_object_rels_per_type
            .get_mut(&o.object_type)
            .unwrap();
        for rels in get_object_relationships(o) {
            match object_map.get(&rels.object_id) {
                Some(rel_obj) => {
                    rels_for_type.insert((rels.qualifier, rel_obj.object_type.clone()));
                }
                None => {
                    eprintln!("Malformed OCEL: Object {} has relationship to object ID {}, which does not belong to any object",o.id, rels.object_id);
                }
            }
        }
    }
    object_to_object_rels_per_type
}

#[derive(Debug, Clone)]
pub struct LinkedOCEL<'a> {
    pub event_map: HashMap<String, &'a OCELEvent>,
    pub object_map: HashMap<String, &'a OCELObject>,
    pub events_of_type: HashMap<String, Vec<&'a OCELEvent>>,
    pub objects_of_type: HashMap<String, Vec<&'a OCELObject>>,
    pub object_events_map: HashMap<String, Vec<String>>,
    pub object_rels_per_type: HashMap<String, HashSet<QualifierAndObjectType>>,
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
    let object_rels_per_type = get_object_rels_per_type(ocel, &object_map);
    LinkedOCEL {
        event_map,
        object_map,
        events_of_type,
        objects_of_type,
        object_events_map,
        object_rels_per_type,
    }
}
