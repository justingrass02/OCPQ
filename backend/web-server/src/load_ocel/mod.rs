use std::{fs::File, io::BufReader};

use axum::{extract::State, http::StatusCode, Json};
use ocedeclare_shared::OCELInfo;
use serde::{Deserialize, Serialize};

use process_mining::event_log::ocel::ocel_struct::OCEL;

use crate::AppState;

#[derive(Deserialize, Serialize)]
pub struct LoadOcel {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OCELFilePath {
    name: &'static str,
    path: &'static str,
}

pub const DEFAULT_OCEL_FILE: &str = "order-management";

pub const OCEL_PATHS: &[OCELFilePath] = &[
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

pub async fn load_ocel_file_req(
    State(state): State<AppState>,
    Json(payload): Json<LoadOcel>,
) -> (StatusCode, Json<Option<OCELInfo>>) {
    match load_ocel_file_to_state(&payload.name, &state) {
        Some(ocel_info) => (StatusCode::OK, Json(Some(ocel_info))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}

pub fn load_ocel_file_to_state(name: &str, state: &AppState) -> Option<OCELInfo> {
    match load_ocel_file(name) {
        Ok(ocel) => {
            let ocel_info: OCELInfo = (&ocel).into();

            let mut x = state.ocel.write().unwrap();
            *x = Some(ocel);
            Some(ocel_info)
        }
        Err(e) => {
            eprintln!("Error importing OCEL: {:?}", e);
            None
        }
    }
}

pub fn load_ocel_file(name: &str) -> Result<OCEL, std::io::Error> {
    match OCEL_PATHS.iter().find(|op| op.name == name) {
        Some(ocel_path) => {
            let file = File::open(ocel_path.path)?;
            let reader = BufReader::new(file);
            let ocel: OCEL = serde_json::from_reader(reader)?;
            Ok(ocel)
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "OCEL with that name not found",
        )),
    }
}
