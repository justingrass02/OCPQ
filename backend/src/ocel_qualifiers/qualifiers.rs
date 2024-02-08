use std::collections::{HashMap, HashSet};

use axum::{extract::State, http::StatusCode, Json};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    preprocessing::preprocess::{get_object_rels_per_type, link_ocel_info},
    with_ocel_from_state, AppState,
};

#[derive(Serialize, Deserialize)]
pub struct QualifiersForEventType {
    qualifier: String,
    multiple: bool,
    object_types: Vec<String>,
    counts: Vec<i32>,
}
pub type QualifierAndObjectType = (String, String);

pub async fn get_qualifers_for_object_types(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashSet<QualifierAndObjectType>>>>,
) {
    match with_ocel_from_state(&State(state), |ocel| {
        let x = link_ocel_info(ocel);
        return x.object_rels_per_type;
    }) {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}
pub async fn get_qualifiers_for_event_types(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashMap<String, QualifiersForEventType>>>>,
) {
    match with_ocel_from_state(
        &State(state),
        |ocel| -> HashMap<String, HashMap<String, QualifiersForEventType>> {
            let qualifiers_per_event_type: Vec<(
                String,
                HashMap<QualifierAndObjectType, Vec<i32>>,
            )> = ocel
                .event_types
                .par_iter()
                .map(|et| {
                    (
                        et.name.clone(),
                        ocel.events
                            .iter()
                            .filter(|ev| ev.event_type == et.name)
                            .map(|ev| match ev.relationships.as_ref() {
                                Some(rs) => rs
                                    .iter()
                                    .filter_map(|r| {
                                        let obj = ocel.objects.iter().find(|o| o.id == r.object_id);
                                        obj.map(|obj| {
                                            (r.qualifier.clone(), obj.object_type.clone())
                                        })
                                    })
                                    .fold(HashMap::new(), |mut acc, c| {
                                        *acc.entry(c).or_insert(0) += 1;
                                        acc
                                    }),
                                None => HashMap::new(),
                            })
                            .fold(HashMap::new(), |mut acc, c| {
                                c.into_iter().for_each(|(a, b)| {
                                    acc.entry(a).or_insert(Vec::new()).push(b);
                                });
                                acc
                            }),
                    )
                })
                .collect();
            qualifiers_per_event_type
                .into_iter()
                .map(|(event_type, quals)| {
                    let mut ret: HashMap<String, QualifiersForEventType> = HashMap::new();
                    quals.iter().for_each(|((qualifier, obj_type), counts)| {
                        match ret.get_mut(qualifier) {
                            Some(pre_val) => {
                                if !pre_val.object_types.contains(obj_type) {
                                    pre_val.object_types.push(obj_type.clone());
                                }
                                for c in counts {
                                    if *c > 0 {
                                        pre_val.multiple = true;
                                    }
                                    pre_val.counts.push(*c);
                                }
                            }
                            None => {
                                ret.insert(
                                    qualifier.clone(),
                                    QualifiersForEventType {
                                        qualifier: qualifier.clone(),
                                        multiple: counts.iter().any(|c| *c > 1),
                                        object_types: vec![obj_type.clone()],
                                        counts: counts.clone(),
                                    },
                                );
                            }
                        }
                    });

                    (event_type, ret)
                })
                .collect()
        },
    ) {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}
