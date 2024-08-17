use process_mining::{ocel::ocel_struct::OCELType, OCEL};
use serde::{Deserialize, Serialize};

pub mod ocel_qualifiers {
    pub mod qualifiers;
}
pub mod binding_box;
pub mod constraints_2;
pub mod discovery;
pub mod ocel_graph;
pub mod preprocessing {
    pub mod linked_ocel;
    pub mod preprocess;
    pub mod tests;
}
pub mod cel;

#[derive(Debug, Serialize, Deserialize)]
pub struct OCELInfo {
    pub num_objects: usize,
    pub num_events: usize,
    pub object_types: Vec<OCELType>,
    pub event_types: Vec<OCELType>,
    pub object_ids: Vec<String>,
    pub event_ids: Vec<String>,
}

impl From<&OCEL> for OCELInfo {
    fn from(val: &OCEL) -> Self {
        OCELInfo {
            num_objects: val.objects.len(),
            num_events: val.events.len(),
            object_types: val.object_types.clone(),
            event_types: val.event_types.clone(),
            event_ids: val.events.iter().map(|ev| ev.id.clone()).collect(),
            object_ids: val.objects.iter().map(|ob| ob.id.clone()).collect(),
        }
    }
}
