use std::{
    collections::{HashMap, HashSet}, env, sync::{Arc, RwLock}
};

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use itertools::Itertools;

use ocedeclare_shared::{constraints::{check_with_tree, CheckWithTreeRequest, ViolationsWithoutID}, discovery::{auto_discover_count_constraints, auto_discover_eventually_follows, auto_discover_or_constraints, get_obj_types_per_ev_type, AutoDiscoverConstraintsRequest, AutoDiscoverConstraintsResponse}, ocel_qualifiers::qualifiers::{get_qualifiers_for_event_types, QualifierAndObjectType, QualifiersForEventType}, preprocessing::preprocess::link_ocel_info};
use process_mining::event_log::ocel::ocel_struct::{OCELType, OCEL};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::load_ocel::{get_available_ocels, load_ocel_file_req, load_ocel_file_to_state, DEFAULT_OCEL_FILE};
pub mod load_ocel;

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
        .route(
            "/ocel/event-qualifiers",
            get(get_qualifiers_for_event_types_handler),
        )
        .route(
            "/ocel/object-qualifiers",
            get(get_qualifers_for_object_types),
        )
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

impl From<&OCEL> for OCELInfo {
    fn from(val: &OCEL) -> Self {
        OCELInfo {
            num_objects: val.objects.len(),
            num_events: val.events.len(),
            object_types: val.object_types.clone(),
            event_types: val.event_types.clone(),
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



pub async fn get_qualifiers_for_event_types_handler(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashMap<String, QualifiersForEventType>>>>,
) {
    match with_ocel_from_state(
        &State(state),
        |ocel| -> HashMap<String, HashMap<String, QualifiersForEventType>> {
            get_qualifiers_for_event_types(ocel)
        },
    ) {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}



pub async fn get_qualifers_for_object_types(
    State(state): State<AppState>,
) -> (
    StatusCode,
    Json<Option<HashMap<String, HashSet<QualifierAndObjectType>>>>,
) {
    let qualifier_and_type = with_ocel_from_state(&State(state), |ocel| {
        let x = link_ocel_info(ocel);
        x.object_rels_per_type
    });
    match qualifier_and_type {
        Some(x) => (StatusCode::OK, Json(Some(x))),
        None => (StatusCode::BAD_REQUEST, Json(None)),
    }
}

pub async fn check_with_tree_req(
    state: State<AppState>,
    Json(req): Json<CheckWithTreeRequest>,
) -> (
    StatusCode,
    Json<Option<(Vec<usize>, Vec<ViolationsWithoutID>)>>,
) {
    with_ocel_from_state(&state, |ocel| {
        (
            StatusCode::OK,
            Json(Some(check_with_tree(req.variables, req.nodes_order, ocel))),
        )
    })
    .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, Json(None)))
}

pub async fn auto_discover_constraints_handler(
    state: State<AppState>,
    Json(req): Json<AutoDiscoverConstraintsRequest>,
) -> Json<Option<AutoDiscoverConstraintsResponse>> {
    Json(with_ocel_from_state(&state, |ocel| {
        let linked_ocel = link_ocel_info(ocel);
        let obj_types_per_ev_type = get_obj_types_per_ev_type(&linked_ocel);
        let count_constraints = match req.count_constraints {
            Some(count_options) => auto_discover_count_constraints(
                ocel,
                &obj_types_per_ev_type,
                &linked_ocel,
                None,
                count_options,
            ),
            None => Vec::new(),
        };
        let eventually_follows_constraints = match req.eventually_follows_constraints {
            Some(eventually_follows_options) => {
                auto_discover_eventually_follows(&linked_ocel, None, eventually_follows_options)
            }
            None => Vec::new(),
        };
        let or_constraints = match req.or_constraints {
            Some(or_constraint_option) => auto_discover_or_constraints(
                ocel,
                &linked_ocel,
                &obj_types_per_ev_type,
                or_constraint_option,
            ),
            None => Vec::new(),
        };
        AutoDiscoverConstraintsResponse {
            count_constraints: count_constraints
                .into_iter()
                .map(|c| c.constraint)
                .collect(),
            eventually_follows_constraints: eventually_follows_constraints
                .into_iter()
                .map(|efc| efc.constraint)
                .collect(),
            or_constraints,
        }
    }))
}
