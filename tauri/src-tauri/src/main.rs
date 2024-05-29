// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use ocedeclare_shared::{
    binding_box::{evaluate_box_tree, CheckWithBoxTreeRequest, EvaluateBoxTreeResult},
    discovery::{
        auto_discover_constraints_with_options, AutoDiscoverConstraintsRequest,
        AutoDiscoverConstraintsResponse,
    },
    ocel_graph::{get_ocel_graph, OCELGraph, OCELGraphOptions},
    ocel_qualifiers::qualifiers::{get_qualifiers_for_event_types, QualifiersForEventType},
    preprocessing::linked_ocel::{link_ocel_info, IndexLinkedOCEL},
    OCELInfo,
};
use process_mining::{import_ocel_json_from_path, import_ocel_xml_file};
use tauri::State;

type OCELStore = Mutex<Option<IndexLinkedOCEL>>;

#[tauri::command(async)]
fn import_ocel(path: &str, state: tauri::State<OCELStore>) -> Result<OCELInfo, String> {
    let ocel = match path.ends_with(".json") {
        true => import_ocel_json_from_path(path).map_err(|e| format!("{:?}", e))?,
        false => import_ocel_xml_file(path),
    };
    let ocel_info: OCELInfo = (&ocel).into();
    let mut state_guard = state.lock().unwrap();
    *state_guard = Some(link_ocel_info(ocel));
    Ok(ocel_info)
}

#[tauri::command(async)]
fn get_current_ocel_info(state: tauri::State<OCELStore>) -> Result<OCELInfo, String> {
    let res: Result<OCELInfo, String> = match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok((&ocel.ocel).into()),
        None => Err("No OCEL loaded".to_string()),
    };
    res
}

#[tauri::command(async)]
fn get_event_qualifiers(
    state: State<OCELStore>,
) -> Result<HashMap<String, HashMap<String, QualifiersForEventType>>, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(get_qualifiers_for_event_types(&ocel.ocel)),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn get_object_qualifiers(
    state: State<OCELStore>,
) -> Result<HashMap<String, HashSet<(String, String)>>, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(ocel.object_rels_per_type.clone()),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn check_with_box_tree(
    req: CheckWithBoxTreeRequest,
    state: State<OCELStore>,
) -> Result<EvaluateBoxTreeResult, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(evaluate_box_tree(req.tree, &ocel)),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn auto_discover_constraints(
    options: AutoDiscoverConstraintsRequest,
    state: State<OCELStore>,
) -> Result<AutoDiscoverConstraintsResponse, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(auto_discover_constraints_with_options(&ocel.ocel, options)),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn ocel_graph(options: OCELGraphOptions, state: State<OCELStore>) -> Result<OCELGraph, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => match get_ocel_graph(&ocel, options) {
            Some(graph) => Ok(graph),
            None => Err("Could not construct OCEL Graph".to_string()),
        },
        None => Err("No OCEL loaded".to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(OCELStore::new(None))
        .invoke_handler(tauri::generate_handler![
            import_ocel,
            get_current_ocel_info,
            get_event_qualifiers,
            get_object_qualifiers,
            check_with_box_tree,
            auto_discover_constraints,
            ocel_graph
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
