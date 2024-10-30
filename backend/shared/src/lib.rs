use std::collections::HashSet;

use binding_box::EvaluationResultWithCount;
use itertools::Itertools;
use preprocessing::linked_ocel::IndexLinkedOCEL;
use process_mining::{
    ocel::ocel_struct::{OCELEvent, OCELObject, OCELType},
    OCEL,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum IndexOrID {
    #[serde(rename = "id")]
    ID(String),
    #[serde(rename = "index")]
    Index(usize),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectWithIndex {
    pub object: OCELObject,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventWithIndex {
    pub event: OCELEvent,
    pub index: usize,
}

pub fn get_event_info(ocel: &IndexLinkedOCEL, req: IndexOrID) -> Option<EventWithIndex> {
    let ev_with_index = match req {
        IndexOrID::ID(id) => ocel
            .ev_by_id(&id)
            .cloned()
            .and_then(|ev| ocel.index_of_ev(&id).map(|ev_index| (ev, ev_index.0))),
        IndexOrID::Index(index) => ocel.ocel.events.get(index).cloned().map(|ev| (ev, index)),
    };
    ev_with_index.map(|(event, index)| EventWithIndex { event, index })
}

pub fn get_object_info(ocel: &IndexLinkedOCEL, req: IndexOrID) -> Option<ObjectWithIndex> {
    let ob_with_index = match req {
        IndexOrID::ID(id) => ocel
            .ob_by_id(&id)
            .cloned()
            .and_then(|ob| ocel.index_of_ob(&id).map(|ob_index| (ob, ob_index.0))),
        IndexOrID::Index(index) => ocel.ocel.objects.get(index).cloned().map(|ev| (ev, index)),
    };
    ob_with_index.map(|(object, index)| ObjectWithIndex { object, index })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
pub struct TableExportOptions {
    include_violation_status: bool,
    include_ids: bool,
    omit_header: bool,
    labels: Vec<String>,
}
impl Default for TableExportOptions {
    fn default() -> Self {
        Self {
            include_violation_status: true,
            include_ids: true,
            omit_header: false,
            labels: Vec::default(),
        }
    }
}

pub fn export_bindings_to_csv_writer<W: std::io::Write>(
    ocel: &IndexLinkedOCEL,
    bindings: &EvaluationResultWithCount,
    writer: &mut W,
    options: &TableExportOptions,
) -> Result<(), csv::Error> {
    let mut w = csv::WriterBuilder::new().from_writer(writer);
    if let Some((b, _)) = bindings.situations.first() {
        let ev_vars = b.event_map.keys().sorted().collect_vec();
        let ob_vars = b.object_map.keys().sorted().collect_vec();

        let ev_attrs = ev_vars
            .iter()
            .map(|ev_var| {
                bindings
                    .situations
                    .iter()
                    .flat_map(|(b, _)| {
                        b.get_ev(ev_var, ocel)
                            .iter()
                            .flat_map(|e| e.attributes.iter().map(|a| &a.name))
                            .collect::<Vec<_>>()
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect_vec()
            })
            .collect_vec();

        let ob_attrs = ob_vars
            .iter()
            .map(|ob_var| {
                bindings
                    .situations
                    .iter()
                    .flat_map(|(b, _)| {
                        b.get_ob(ob_var, ocel)
                            .iter()
                            .flat_map(|e| e.attributes.iter().map(|a| &a.name))
                            .collect::<Vec<_>>()
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect_vec()
            })
            .collect_vec();
        // Write Headers
        if !options.omit_header {
            // First object/event ID, then attributes, then next object/event ID, ..
            for (ob, ob_attrs) in ob_vars.iter().zip(&ob_attrs) {
                if options.include_ids {
                    w.write_field(format!("o{}", ob.0 + 1))?;
                }
                for attr in ob_attrs {
                    w.write_field(format!("o{}.{}", ob.0 + 1, attr))?;
                }
            }
            for (ev, ev_attrs) in ev_vars.iter().zip(&ev_attrs) {
                if options.include_ids {
                    w.write_field(format!("e{}", ev.0 + 1))?;
                }
                for attr in ev_attrs {
                    w.write_field(format!("e{}.{}", ev.0 + 1, attr))?;
                }
            }

            for label in &options.labels {
                w.write_field(label)?;
            }

            if options.include_violation_status {
                w.write_field("Violated")?;
            }
            w.write_record(None::<&[u8]>)?;
        }

        for (b, v) in &bindings.situations {
            for (ob_v, ob_attrs) in ob_vars.iter().zip(&ob_attrs) {
                if let Some(ob) = b.get_ob(ob_v, ocel) {
                    if options.include_ids {
                        w.write_field(&ob.id)?;
                    }
                    for attr in ob_attrs {
                        if let Some(val) = ob
                            .attributes
                            .iter()
                            .filter(|a| &&a.name == attr)
                            .sorted_by_key(|a| a.time)
                            .next()
                        {
                            w.write_field(format!("{}", val.value))?;
                        } else {
                            w.write_field("")?;
                        }
                    }
                } else {
                    if options.include_ids {
                        w.write_field("")?;
                    }
                    for _attr in ob_attrs {
                        w.write_field("")?;
                    }
                }
            }
            for (ev_v, ev_attrs) in ev_vars.iter().zip(&ev_attrs) {
                if let Some(ev) = b.get_ev(ev_v, ocel) {
                    if options.include_ids {
                        w.write_field(&ev.id)?;
                    }
                    for attr in ev_attrs {
                        if let Some(val) = ev.attributes.iter().find(|a| &&a.name == attr) {
                            w.write_field(format!("{}", val.value))?;
                        } else {
                            w.write_field("")?;
                        }
                    }
                } else {
                    if options.include_ids {
                        w.write_field("")?;
                    }
                    for _attr in ev_attrs {
                        w.write_field("")?;
                    }
                }
            }

            for label in &options.labels {
                match b.label_map.get(label) {
                    Some(val) => w.write_field(val.to_string())?,
                    None => w.write_field("null")?,
                }
            }

            if options.include_violation_status {
                w.write_field(format!("{}", v.is_some()))?;
            }
            w.write_record(None::<&[u8]>)?;
        }
        // w.write_record(ob_vars.iter().map(|ob| vec![format!("o{}", ob.0)].into).chain(
        //    ,
        // ).chain(ev_vars.iter().map(|ob| format!("o{}", ob.0)).chain(
        //     ev_vars.iter().zip(ev_attrs).flat_map(|(ob, attrs)| {
        //         attrs.into_iter().map(|attr| format!("o{}.{}", ob.0, attr))
        //     }),
        // )))?;
    }
    Ok(())
}
