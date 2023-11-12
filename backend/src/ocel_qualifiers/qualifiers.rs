use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, Json};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{with_ocel_from_state, AppState};

#[derive(Serialize, Deserialize)]
pub struct QualifiersForEventType {
    qualifier: String,
    multiple: bool,
    object_types: Vec<String>,
    counts: Vec<i32>,
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
            let qualifiers_per_event_type: Vec<(String, HashMap<(String, String), Vec<i32>>)> =
                ocel.event_types
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
                                            let obj =
                                                ocel.objects.iter().find(|o| o.id == r.object_id);
                                            match obj {
                                                Some(obj) => Some((
                                                    r.qualifier.clone(),
                                                    obj.object_type.clone(),
                                                )),
                                                None => None,
                                            }
                                        })
                                        .fold(HashMap::new(), |mut acc, c| {
                                            *acc.entry(c).or_insert(0) += 1;
                                            acc
                                        }),
                                    None => return HashMap::new(),
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
            return qualifiers_per_event_type
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
                .collect();
        },
    ) {
        Some(x) => return (StatusCode::OK, Json(Some(x))),
        None => return (StatusCode::BAD_REQUEST, Json(None)),
    }
}
