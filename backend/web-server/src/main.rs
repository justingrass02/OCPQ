use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{header::CONTENT_TYPE, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{Arc, RwLock},
};

use ocedeclare_shared::{
    binding_box::{
        evaluate_box_tree,
        structs::{BindingBoxTree, EvaluationResults},
    },
    constraints::{check_with_tree, CheckWithTreeRequest, ViolationsWithoutID},
    discovery::{
        auto_discover_constraints_with_options, AutoDiscoverConstraintsRequest,
        AutoDiscoverConstraintsResponse,
    },
    ocel_qualifiers::qualifiers::{
        get_qualifiers_for_event_types, QualifierAndObjectType, QualifiersForEventType,
    },
    preprocessing::{linked_ocel::IndexLinkedOCEL, preprocess::link_ocel_info},
    OCELInfo,
};
use process_mining::{event_log::ocel::ocel_struct::OCEL, import_ocel_xml_slice};
use tower_http::cors::CorsLayer;

use crate::load_ocel::{
    get_available_ocels, load_ocel_file_req, load_ocel_file_to_state, DEFAULT_OCEL_FILE,
};
pub mod load_ocel;

#[derive(Clone)]
pub struct AppState {
    ocel: Arc<RwLock<Option<IndexLinkedOCEL>>>,
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
        .route(
            "/ocel/upload-json",
            post(upload_ocel_json).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/ocel/upload-xml",
            post(upload_ocel_xml).layer(DefaultBodyLimit::disable()),
        )
        .route("/ocel/available", get(get_available_ocels))
        .route(
            "/ocel/event-qualifiers",
            get(get_qualifiers_for_event_types_handler),
        )
        .route(
            "/ocel/object-qualifiers",
            get(get_qualifers_for_object_types),
        )
        .route("/ocel/check-constraints-box", post(check_with_box_tree_req))
        .route("/ocel/check-constraints", post(check_with_tree_req))
        .route(
            "/ocel/discover-constraints",
            post(auto_discover_constraints_handler),
        )
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
    match with_ocel_from_state(&State(state), |ocel| (&ocel.ocel).into()) {
        Some(ocel_info) => (StatusCode::OK, Json(Some(ocel_info))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

async fn upload_ocel_xml<'a>(
    State(state): State<AppState>,
    ocel_bytes: Bytes,
) -> (StatusCode, Json<OCELInfo>) {
    let ocel = import_ocel_xml_slice(&ocel_bytes);
    let mut x = state.ocel.write().unwrap();
    let ocel_info: OCELInfo = (&ocel).into();
    *x = Some(IndexLinkedOCEL::new(ocel));

    (StatusCode::OK, Json(ocel_info))
}

async fn upload_ocel_json<'a>(
    State(state): State<AppState>,
    ocel_bytes: Bytes,
) -> (StatusCode, Json<OCELInfo>) {
    let ocel: OCEL = serde_json::from_slice(&ocel_bytes).unwrap();
    let mut x = state.ocel.write().unwrap();
    let ocel_info: OCELInfo = (&ocel).into();
    *x = Some(IndexLinkedOCEL::new(ocel));
    (StatusCode::OK, Json(ocel_info))
}

pub fn with_ocel_from_state<T, F>(State(state): &State<AppState>, f: F) -> Option<T>
where
    F: FnOnce(&IndexLinkedOCEL) -> T,
{
    let read_guard = state.ocel.read().ok()?;
    let ocel_ref = read_guard.as_ref()?;
    Some(f(ocel_ref))
}

pub async fn get_qualifiers_for_event_types_handler<'a>(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashMap<String, QualifiersForEventType>>>>,
) {
    match with_ocel_from_state(
        &State(state),
        |ocel| -> HashMap<String, HashMap<String, QualifiersForEventType>> {
            get_qualifiers_for_event_types(&ocel.ocel)
        },
    ) {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}

pub async fn get_qualifers_for_object_types<'a>(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashSet<QualifierAndObjectType>>>>,
) {
    let qualifier_and_type = with_ocel_from_state(&State(state), |ocel| {
        link_ocel_info(&ocel.ocel).object_rels_per_type.clone()
    });
    match qualifier_and_type {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}

pub async fn check_with_tree_req<'a>(
    state: State<AppState>,
    Json(req): Json<CheckWithTreeRequest>,
) -> (
    StatusCode,
    Json<Option<(Vec<usize>, Vec<ViolationsWithoutID>)>>,
) {
    with_ocel_from_state(&state, |ocel| {
        (
            StatusCode::OK,
            Json(Some(check_with_tree(
                req.variables,
                req.nodes_order,
                &ocel.ocel,
            ))),
        )
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}

#[derive(Serialize, Deserialize)]
pub struct CheckWithBoxTreeRequest {
    pub tree: BindingBoxTree,
}

pub async fn check_with_box_tree_req<'a>(
    state: State<AppState>,
    Json(req): Json<CheckWithBoxTreeRequest>,
) -> (StatusCode, Json<Option<EvaluationResults>>) {
    with_ocel_from_state(&state, |ocel| {
        (
            StatusCode::OK,
            Json(Some(evaluate_box_tree(req.tree, &ocel))),
        )
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}

pub async fn auto_discover_constraints_handler<'a>(
    state: State<AppState>,
    Json(req): Json<AutoDiscoverConstraintsRequest>,
) -> Json<Option<AutoDiscoverConstraintsResponse>> {
    Json(with_ocel_from_state(&state, |ocel| {
        auto_discover_constraints_with_options(&ocel.ocel, req)
    }))
}
