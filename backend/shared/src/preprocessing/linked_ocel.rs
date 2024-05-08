use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;
use process_mining::{
    event_log::ocel::ocel_struct::{OCELEvent, OCELObject, OCELRelationship},
    OCEL,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ocel_qualifiers::qualifiers::QualifierAndObjectType;

pub fn get_object_events_map(
    ocel: &OCEL,
    object_map: &HashMap<String, ObjectIndex>,
) -> HashMap<ObjectIndex, Vec<EventIndex>> {
    let mut object_events_map: HashMap<ObjectIndex, Vec<EventIndex>> = ocel
        .objects
        .iter()
        .enumerate()
        .map(|(index, _)| (ObjectIndex(index), Vec::new()))
        .collect();

    for (e_index, e) in ocel.events.iter().enumerate() {
        let rels = get_event_relationships(e);
        for r in rels {
            match object_map
                .get(&r.object_id)
                .and_then(|ob_index| object_events_map.get_mut(ob_index))
            {
                Some(o_evs) => o_evs.push(EventIndex(e_index)),
                None => {
                    eprintln!("Malformed OCEL: Event {} relates to object ID {}, which does not belong to any object.",e.id,r.object_id)
                }
            }
        }
    }
    object_events_map
}

pub fn get_events_of_type_associated_with_objects<'a>(
    linked_ocel: &'a IndexLinkedOCEL,
    event_types: &HashSet<std::string::String>,
    object_indices: &[ObjectIndex],
) -> Vec<EventIndex> {
    if object_indices.is_empty() {
        return linked_ocel
            .ocel
            .events
            .iter()
            .enumerate()
            .filter(|(_e_index, e)| event_types.contains(&e.event_type))
            .map(|(e_index, _)| EventIndex(e_index))
            .collect();
    }
    let mut sorted_object_ids_iter = object_indices.iter().sorted_by(|a, b| {
        linked_ocel
            .object_events_map
            .get(*a)
            .unwrap()
            .len()
            .cmp(&linked_ocel.object_events_map.get(*b).unwrap().len())
    });

    let mut intersection: HashSet<EventIndex> = linked_ocel
        .object_events_map
        .get(sorted_object_ids_iter.next().unwrap())
        .unwrap()
        .iter()
        .filter(|e_index| {
            event_types.contains(&linked_ocel.ev_by_index(&e_index).unwrap().event_type)
        })
        .copied()
        .collect();
    for other in sorted_object_ids_iter {
        let other_map: HashSet<EventIndex> = linked_ocel
            .object_events_map
            .get(other)
            .unwrap()
            .iter()
            .copied()
            .collect();
        intersection.retain(|ev| other_map.contains(ev))
    }
    return intersection.into_iter().collect();
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

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventIndex(pub usize);

impl Display for EventIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ev:{}", self.0)
    }
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectIndex(pub usize);
impl Display for ObjectIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ob:{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct IndexLinkedOCEL {
    pub object_events_map: HashMap<ObjectIndex, Vec<EventIndex>>,
    pub object_rels_per_type: HashMap<String, HashSet<QualifierAndObjectType>>,

    pub events_of_type: HashMap<String, Vec<EventIndex>>,
    pub objects_of_type: HashMap<String, Vec<ObjectIndex>>,
    pub ocel: OCEL,
    pub event_index_map: HashMap<String, EventIndex>,
    pub object_index_map: HashMap<String, ObjectIndex>,
}

impl IndexLinkedOCEL {
    pub fn new(ocel: OCEL) -> Self {
        link_ocel_info(ocel)
    }
    pub fn ev_by_index<'a>(&'a self, index: &EventIndex) -> Option<&'a OCELEvent> {
        self.ocel.events.get(index.0)
    }
    pub fn ob_by_index<'a>(&'a self, index: &ObjectIndex) -> Option<&'a OCELObject> {
        self.ocel.objects.get(index.0)
    }

    pub fn index_of_ob<'a>(&'a self, ob_id: &String) -> Option<&'a ObjectIndex> {
        self.object_index_map.get(ob_id)
    }

    pub fn ob_by_id<'a>(&'a self, ob_id: &String) -> Option<&'a OCELObject> {
        self.index_of_ob(ob_id)
            .and_then(|ob_index| self.ob_by_index(ob_index))
    }

    pub fn index_of_ev<'a>(&'a self, ev_id: &String) -> Option<&'a EventIndex> {
        self.event_index_map.get(ev_id)
    }
}

pub fn link_ocel_info(ocel: OCEL) -> IndexLinkedOCEL {
    let object_map: HashMap<String, &OCELObject> = ocel
        .objects
        .iter()
        .map(|obj| (obj.id.clone(), obj))
        .collect();

    let events_of_type = ocel
        .event_types
        .iter()
        .map(|ev_type| {
            (
                ev_type.name.clone(),
                ocel.events
                    .iter()
                    .enumerate()
                    .filter(|(_index, ev)| ev.event_type == ev_type.name)
                    .map(|(index, _)| EventIndex(index))
                    .collect(),
            )
        })
        .collect();

    let objects_of_type = ocel
        .object_types
        .iter()
        .map(|obj_type| {
            (
                obj_type.name.clone(),
                ocel.objects
                    .iter()
                    .enumerate()
                    .filter(|(_index, ob)| ob.object_type == obj_type.name)
                    .map(|(index, _)| ObjectIndex(index))
                    .collect(),
            )
        })
        .collect();

    let event_index_map = ocel
        .events
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id.clone(), EventIndex(i)))
        .collect();
    let object_index_map = ocel
        .objects
        .iter()
        .enumerate()
        .map(|(i, o)| (o.id.clone(), ObjectIndex(i)))
        .collect();
    let object_events_map = get_object_events_map(&ocel, &object_index_map);
    let object_rels_per_type = get_object_rels_per_type(&ocel, &object_map);
    IndexLinkedOCEL {
        events_of_type,
        objects_of_type,
        object_events_map,
        object_rels_per_type,
        ocel,
        event_index_map,
        object_index_map,
    }
}
