// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use ocedeclare_shared::{
    constraints::{check_with_tree, ObjectVariable, TreeNode, ViolationsWithoutID},
    discovery::{
        auto_discover_constraints_with_options, AutoDiscoverConstraintsRequest,
        AutoDiscoverConstraintsResponse,
    },
    ocel_qualifiers::qualifiers::{get_qualifiers_for_event_types, QualifiersForEventType},
    preprocessing::preprocess::link_ocel_info,
    OCELInfo,
};
use process_mining::{import_ocel_json_from_path, import_ocel_xml_file, OCEL};
use tauri::State;

type OCELStore = Mutex<Option<OCEL>>;

#[tauri::command(async)]
fn import_ocel(path: &str, state: tauri::State<OCELStore>) -> Result<OCELInfo, String> {
    let ocel = match path.ends_with(".json") {
        true => import_ocel_json_from_path(path).map_err(|e| format!("{:?}", e))?,
        false => import_ocel_xml_file(path),
    };
    let ocel_info: OCELInfo = (&ocel).into();
    let mut state_guard = state.lock().unwrap();
    *state_guard = Some(ocel);
    Ok(ocel_info)
}

#[tauri::command(async)]
fn get_current_ocel_info(state: tauri::State<OCELStore>) -> Result<OCELInfo, String> {
    let res: Result<OCELInfo, String> = match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(ocel.into()),
        None => Err("No OCEL loaded".to_string()),
    };
    res
}

#[tauri::command(async)]
fn get_event_qualifiers(
    state: State<OCELStore>,
) -> Result<HashMap<String, HashMap<String, QualifiersForEventType>>, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(get_qualifiers_for_event_types(ocel)),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn get_object_qualifiers(
    state: State<OCELStore>,
) -> Result<HashMap<String, HashSet<(String, String)>>, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(link_ocel_info(ocel).object_rels_per_type),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn check_constraint_with_tree(
    variables: Vec<ObjectVariable>,
    nodes: Vec<TreeNode>,
    state: State<OCELStore>,
) -> Result<(Vec<usize>, Vec<ViolationsWithoutID>), String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(check_with_tree(variables, nodes, ocel)),
        None => Err("No OCEL loaded".to_string()),
    }
}

#[tauri::command(async)]
fn auto_discover_constraints(
    options: AutoDiscoverConstraintsRequest,
    state: State<OCELStore>,
) -> Result<AutoDiscoverConstraintsResponse, String> {
    match state.lock().unwrap().as_ref() {
        Some(ocel) => Ok(auto_discover_constraints_with_options(ocel, options)),
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
            check_constraint_with_tree,
            auto_discover_constraints
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
