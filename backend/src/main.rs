use std::{env, sync::{Arc, RwLock}};

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
mod load_ocel {
    pub mod load_ocel;
}
mod ocel_qualifiers {
    pub mod qualifiers;
}
mod constraint_checker {
    pub mod check;
}
mod preprocessing {
    pub mod preprocess;
    mod tests;
}

use constraint_checker::check::check_with_tree_req;
use itertools::Itertools;
use load_ocel::load_ocel::{
    get_available_ocels, load_ocel_file_req, DEFAULT_OCEL_FILE,
};
use ocel_qualifiers::qualifiers::get_qualifiers_for_event_types;
use process_mining::event_log::ocel::ocel_struct::{OCELType, OCEL};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::load_ocel::load_ocel::load_ocel_file_to_state;

#[derive(Clone)]
pub struct AppState {
    ocel: Arc<RwLock<Option<OCEL>>>,
}

#[tokio::main]
async fn main() {
    let args = env::args().collect_vec();
    dbg!(args);
    let state = AppState {
        ocel: Arc::new(RwLock::new(None)),
    };
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_origin(tower_http::cors::Any);

    load_ocel_file_to_state(DEFAULT_OCEL_FILE, &state);

    // build our application with a single route
    let app = Router::new()
        .route("/ocel/load", post(load_ocel_file_req))
        .route("/ocel/info", get(get_loaded_ocel_info))
        .route("/ocel/available", get(get_available_ocels))
        .route("/ocel/qualifiers", get(get_qualifiers_for_event_types))
        .route("/ocel/check-constraints", post(check_with_tree_req))
        .with_state(state)
        .route("/", get(|| async { "Hello, Aaron!" }))
        .layer(cors);
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn get_loaded_ocel_info(
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<OCELInfo>>) {
    match with_ocel_from_state(&State(state), |ocel| ocel.into()) {
        Some(ocel_info) => (StatusCode::OK, Json(Some(ocel_info))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCELInfo {
    pub num_objects: usize,
    pub num_events: usize,
    pub object_types: Vec<OCELType>,
    pub event_types: Vec<OCELType>,
}

impl Into<OCELInfo> for &OCEL {
    fn into(self) -> OCELInfo {
        OCELInfo {
            num_objects: self.objects.len(),
            num_events: self.events.len(),
            object_types: self.object_types.clone(),
            event_types: self.event_types.clone(),
        }
    }
}

pub fn with_ocel_from_state<T, F>(State(state): &State<AppState>, f: F) -> Option<T>
where
    F: FnOnce(&OCEL) -> T,
{
    let read_guard = state.ocel.read().ok()?;
    let ocel_ref = read_guard.as_ref()?;
    Some(f(ocel_ref))
}
