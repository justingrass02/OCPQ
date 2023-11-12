use std::{fs::File, io::BufReader};

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use pm_rust::event_log::ocel::ocel_struct::OCEL;

use crate::{AppState, OCELInfo};

#[derive(Deserialize, Serialize)]
pub struct LoadOcel {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OCELFilePath {
    name: &'static str,
    path: &'static str,
}

const OCEL_PATHS: &'static [OCELFilePath] = &[
    OCELFilePath {
        name: "ContainerLogistics",
        path: "./data/ContainerLogistics.json",
    },
    OCELFilePath {
        name: "ocel2-p2p",
        path: "./data/ocel2-p2p.json",
    },
    OCELFilePath {
        name: "order-management",
        path: "./data/order-management.json",
    },
];

pub async fn get_available_ocels() -> (StatusCode, Json<Option<Vec<&'static str>>>) {
    return (
        StatusCode::OK,
        Json(Some(OCEL_PATHS.iter().map(|p| p.name).collect())),
    );
}

pub async fn load_ocel_file(
    State(state): State<AppState>,
    Json(payload): Json<LoadOcel>,
) -> (StatusCode, Json<Option<OCELInfo>>) {
    match OCEL_PATHS.into_iter().find(|op| op.name == payload.name) {
        Some(ocel_path) => {
            let file = File::open(ocel_path.path).unwrap();
            let reader = BufReader::new(file);
            let ocel: OCEL = serde_json::from_reader(reader).unwrap();
            let ocel_info: OCELInfo = (&ocel).into();

            let mut x = state.ocel.write().unwrap();
            *x = Some(ocel);

            (StatusCode::OK, Json(Some(ocel_info)))
        }
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}
