use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Instant,
};

use process_mining::{ocel::ocel_struct::OCELObject, OCEL};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::load_ocel::{load_ocel_file, DEFAULT_OCEL_FILE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct NodeID(usize);

struct Node {
    id: NodeID,
    data: NodeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VariableID(usize);

struct O2ORestriction {
    referencing: VariableID,
    qualifier: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Qualifier(String);
enum NodeData {
    ObjectQuery {
        object_type: String,
        bind_variable: VariableID,
        o2o_restriction: Option<O2ORestriction>,
    },
    AND(NodeID, NodeID),
    OR(NodeID, NodeID),
    EventQuery {
        event_type: HashSet<String>,
        associated_object_vars: Vec<(VariableID, Option<Qualifier>)>,
        assigning_object_vars: Vec<(VariableID, Option<Qualifier>)>,
        out_mode: OutMode,
    },
}

enum OutMode {
    All,
    First,
    Last,
    None,
}

/// Position of object in [`OCEL`]'s  [`OCEL::objects`] object array
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ObjectID(usize);

#[derive(Debug, Clone, Default)]
struct ObjectVariableBinding(HashMap<VariableID, ObjectID>);

#[derive(Debug, Clone)]
struct Binding {
    id: Uuid,
    variable_binding: ObjectVariableBinding,
    /// NodeId and event ID
    past_events: Vec<(NodeID, usize)>,
    parent_binding_id: Option<Uuid>,
}

impl Binding {
    pub fn update_copy(&self, variable: VariableID, value: ObjectID) -> Binding {
        let mut b = self.clone();
        b.id = Uuid::new_v4();
        b.parent_binding_id = Some(self.id);
        b.variable_binding.0.insert(variable, value);
        b
    }

    pub fn update_copy_with_ev(&self, node_id: NodeID, event_id: usize) -> Binding {
        let mut b = self.clone();
        b.id = Uuid::new_v4();
        b.parent_binding_id = Some(self.id);
        b.past_events.push((node_id, event_id));
        b
    }

    pub fn get_var_value(
        &self,
        variable: VariableID,
        parents: &HashMap<Uuid, Binding>,
    ) -> Option<ObjectID> {
        if let Some(val) = self.variable_binding.0.get(&variable) {
            Some(*val)
        } else if let Some(parent_id) = self.parent_binding_id {
            parents
                .get(&parent_id)
                .unwrap()
                .get_var_value(variable, parents)
        } else {
            println!("{:?}", self);
            println!("No parent! {:?}", variable);
            None
        }
    }
}
struct OCELHelper {
    pub ocel: OCEL,
    /// Map `object_type` to set of object iDs with that type
    pub objects_of_type: HashMap<String, HashSet<ObjectID>>,
    /// Map string obect_id to the [`ObjectID`] index
    pub object_id_map: HashMap<String, ObjectID>,
    pub events_for_object: Vec<HashSet<usize>>,
}

impl OCELHelper {
    pub fn new(ocel: OCEL) -> Self {
        let objects_of_type = ocel
            .object_types
            .par_iter()
            .map(|object_type| {
                let objects_of_this_type: HashSet<ObjectID> = ocel
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| o.object_type == object_type.name)
                    .map(|(i, _)| ObjectID(i))
                    .collect();
                (object_type.name.clone(), objects_of_this_type)
            })
            .collect();
        let object_id_map: HashMap<String, ObjectID> = ocel
            .objects
            .iter()
            .enumerate()
            .map(|(i, obj)| (obj.id.clone(), ObjectID(i)))
            .collect();
        let mut events_for_object: Vec<HashSet<usize>> = vec![HashSet::new(); ocel.objects.len()];
        for (i, ev) in ocel.events.iter().enumerate() {
            for rel in ev.relationships.as_ref().unwrap_or(&Vec::new()) {
                let obj_id: ObjectID = *object_id_map
                    .get(&rel.object_id)
                    .expect("Object ID not known");
                events_for_object[obj_id.0].insert(i);
            }
        }
        Self {
            ocel,
            objects_of_type,
            object_id_map,
            events_for_object,
        }
    }

    pub fn get_obj_by_str_id(&self, id: &str) -> &OCELObject {
        &self.ocel.objects[self.object_id_map.get(id).unwrap().0]
    }

    pub fn str_id_to_id(&self, id: &str) -> ObjectID {
        *self.object_id_map.get(id).unwrap()
    }
    pub fn id_to_str_id(&self, id: &ObjectID) -> &String {
        &self.ocel.objects[id.0].id
    }
    pub fn has_event_given_rel(
        &self,
        ev_id: usize,
        object_id: &str,
        qualifier: &Option<Qualifier>,
    ) -> bool {
        self.ocel.events[ev_id]
            .relationships
            .as_ref()
            .is_some_and(|r| {
                r.iter().any(|rel| {
                    if rel.object_id != object_id {
                        return false;
                    }
                    if let Some(q) = qualifier {
                        rel.qualifier == q.0
                    } else {
                        true
                    }
                })
            })
    }
    pub fn get_events_for_objects_and_qualifier<'a, I>(&self, mut objects: I) -> HashSet<usize>
    where
        I: Iterator<Item = (ObjectID, &'a Option<Qualifier>)>,
    {
        if let Some((first_obj_id, first_qual)) = objects.next() {
            let mut evs: HashSet<usize> = self.events_for_object[first_obj_id.0]
                .iter()
                .filter(|e| {
                    self.has_event_given_rel(**e, self.id_to_str_id(&first_obj_id), first_qual)
                })
                .copied()
                .collect();
            for (obj_id, qual) in objects {
                evs = evs
                    .union(
                        &self.events_for_object[obj_id.0]
                            .iter()
                            .filter(|e| {
                                self.has_event_given_rel(
                                    **e,
                                    self.id_to_str_id(&first_obj_id),
                                    qual,
                                )
                            })
                            .copied()
                            .collect(),
                    )
                    .copied()
                    .collect();
            }
            evs
        } else {
            HashSet::new()
        }
    }

    pub fn get_events_for_objects<'a, I>(&self, objects: I) -> HashSet<usize>
    where
        I: Iterator<Item = &'a ObjectID>,
    {
        self.get_events_for_objects_and_qualifier(objects.map(|o| (*o, &None)))
    }
}

fn expand(
    node: &Node,
    ocel: Arc<OCELHelper>,
    binding: &Binding,
    prev_bindings: &HashMap<Uuid, Binding>,
) -> Vec<Binding> {
    match &node.data {
        NodeData::ObjectQuery {
            object_type,
            bind_variable,
            o2o_restriction,
        } => {
            if let Some(o2o_rest) = o2o_restriction {
                let referenced_object_id: ObjectID = binding
                    .get_var_value(o2o_rest.referencing, prev_bindings)
                    .expect("Referencing non-existing object");
                if let Some(rels) = &ocel.ocel.objects[referenced_object_id.0].relationships {
                    return rels
                        .iter()
                        .filter(|rel| {
                            // Qualifier matches
                            o2o_rest.qualifier == rel.qualifier
                            &&
                            // Object type matches
                            &ocel.get_obj_by_str_id(&rel.object_id).object_type
                                == object_type
                        })
                        .map(|rel| {
                            binding.update_copy(*bind_variable, ocel.str_id_to_id(&rel.object_id))
                        })
                        .collect();
                }
                // No O2O relationship but O2O restriction, so... return empty
                Vec::new()
            } else {
                // Bind all objects of type
                ocel.objects_of_type
                    .get(object_type)
                    .unwrap_or(&HashSet::new())
                    .iter()
                    .map(|obj_id| binding.update_copy(*bind_variable, *obj_id))
                    .collect()
            }
        }
        NodeData::EventQuery {
            event_type,
            associated_object_vars,
            assigning_object_vars: _,
            out_mode,
        } => {
            let it = ocel
                .get_events_for_objects_and_qualifier(associated_object_vars.iter().map(
                    |(variable_id, qualifier)| {
                        (
                            binding.get_var_value(*variable_id, prev_bindings).unwrap(),
                            qualifier,
                        )
                    },
                ))
                .into_iter()
                .filter(|ev| event_type.contains(&ocel.ocel.events[*ev].event_type))
                .map(|ev| binding.update_copy_with_ev(node.id, ev));
            let res: Box<dyn Iterator<Item = Binding>> = match out_mode {
                OutMode::All => Box::new(it),
                OutMode::First => Box::new(it.take(1)),
                OutMode::Last => {
                    if let Some(last) = it.last() {
                        Box::new(vec![last].into_iter())
                    } else {
                        Box::new(vec![].into_iter())
                    }
                }
                OutMode::None => Box::new(vec![].into_iter()),
            };
            res.collect()
        }
        // NodeData::AND(_, _) => todo!(),
        // NodeData::OR(_, _) => todo!(),
        _ => todo!("Node Type not implemented yet"),
    }
}

#[test]
fn new_constraints_test() {
    let nodes = vec![
        Node {
            id: NodeID(0),
            data: NodeData::ObjectQuery {
                object_type: "customers".to_string(),
                bind_variable: VariableID(0),
                o2o_restriction: None,
            },
        },
        Node {
            id: NodeID(1),
            data: NodeData::ObjectQuery {
                object_type: "orders".to_string(),
                bind_variable: VariableID(1),
                // o2o_restriction: None,
                o2o_restriction: Some(O2ORestriction {
                    referencing: VariableID(0),
                    qualifier: "places".to_string(),
                }),
            },
        },
        Node {
            id: NodeID(2),
            data: NodeData::ObjectQuery {
                object_type: "orders".to_string(),
                bind_variable: VariableID(2),
                o2o_restriction: Some(O2ORestriction {
                    referencing: VariableID(0),
                    qualifier: "places".to_string(),
                }),
            },
        },
        // Node {
        //     id: NodeID(3),
        //     data: NodeData::ObjectQuery {
        //         object_type: "items".to_string(),
        //         bind_variable: VariableID(3),
        //         o2o_restriction: Some(O2ORestriction {
        //             referencing: VariableID(1),
        //             qualifier: "comprises".to_string(),
        //         }),
        //     },
        // },
        // Node {
        //     id: NodeID(4),
        //     data: NodeData::ObjectQuery {
        //         object_type: "items".to_string(),
        //         bind_variable: VariableID(4),
        //         o2o_restriction: Some(O2ORestriction {
        //             referencing: VariableID(2),
        //             qualifier: "comprises".to_string(),
        //         }),
        //     },
        // },
        Node {
            id: NodeID(42),
            data: NodeData::EventQuery {
                event_type: vec![("confirm order").to_string()].into_iter().collect(),
                associated_object_vars: vec![(VariableID(1), Some(Qualifier("order".to_string())))],
                assigning_object_vars: vec![],
                out_mode: OutMode::All,
            },
        },
    ];
    let ocel = load_ocel_file(DEFAULT_OCEL_FILE).expect("Default OCEL File not available");

    // let all_qualifiers: HashSet<String> = ocel.objects.iter().flat_map(|o| o.relationships.clone().unwrap_or(Vec::new()).iter().map(|r| r.qualifier.clone()).collect::<Vec<_>>()).collect();
    // println!("All O2O Qualifiers: {:?}", all_qualifiers);

    let ocel_helper: Arc<OCELHelper> = Arc::new(OCELHelper::new(ocel));
    let mut bindings: HashMap<Uuid, Binding> = vec![Binding {
        id: Uuid::new_v4(),
        variable_binding: ObjectVariableBinding::default(),
        past_events: Vec::new(),
        parent_binding_id: None,
    }]
    .into_iter()
    .map(|b| (b.id, b))
    .collect();
    let new_bindings_ids: Mutex<Vec<Uuid>> = Mutex::new(bindings.keys().map(|id| *id).collect());
    let now = Instant::now();
    for node in nodes {
        let prev_bindings = bindings.clone();
        let prev_new_bindings_ids = new_bindings_ids.lock().unwrap().clone();
        new_bindings_ids.lock().unwrap().clear();
        bindings = bindings
            .into_par_iter()
            .flat_map(|(b_id, b)| {
                if prev_new_bindings_ids.contains(&b_id) {
                    let mut new_bindings =
                        expand(&node, Arc::clone(&ocel_helper), &b, &prev_bindings);
                    new_bindings_ids
                        .lock()
                        .unwrap()
                        .extend(new_bindings.iter().map(|b| b.id));
                    match &node.data {
                        NodeData::EventQuery {
                            event_type: _,
                            associated_object_vars: _,
                            assigning_object_vars: _,
                            out_mode: _,
                        } => {
                            new_bindings.push(b);
                        }
                        _ => {}
                    }
                    new_bindings
                } else {
                    vec![b]
                }
            })
            .map(|b| (b.id, b))
            .collect();

        println!(
            "Got {} new bindings (Total: {})!",
            new_bindings_ids.lock().unwrap().len(),
            bindings.len()
        );
    }
    println!("Finished in {:?}", now.elapsed());
}
