#[cfg(test)]
#[test]
pub fn test() {
    use std::time::Instant;

    use crate::{
        load_ocel::{load_ocel_file, DEFAULT_OCEL_FILE},
        preprocessing::preprocess::get_object_events_map,
    };

    let mut now = Instant::now();
    let ocel = load_ocel_file(DEFAULT_OCEL_FILE).unwrap();
    println!(
        "Loaded OCEL with {} events and {} objects in {:?}",
        ocel.events.len(),
        ocel.objects.len(),
        now.elapsed()
    );
    now = Instant::now();
    let object_events_map = get_object_events_map(&ocel);
    println!(
        "Constructed object events map (of size {}) in {:?}",
        object_events_map.len(),
        now.elapsed()
    );
}
