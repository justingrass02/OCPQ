use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use process_mining::{
    ocel::ocel_struct::{OCELEvent, OCELObject},
    OCEL,
};

use crate::preprocessing::preprocess::LinkedOCEL;

pub struct BindingBox {
    pub new_event_vars: NewVariables,
    pub new_object_vars: NewVariables,
    pub filter_constraint: Vec<FilterConstraint>,
}

#[derive(Debug, Default, Clone)]
pub struct Binding {
    pub event_map: HashMap<String, String>,
    pub object_map: HashMap<String, String>,
}

impl Binding {
    pub fn expand_with_ev(mut self, ev_var_name: String, ev_id: String) -> Self {
        self.event_map.insert(ev_var_name, ev_id);
        return self;
    }
    pub fn expand_with_ob(mut self, ob_var_name: String, ob_id: String) -> Self {
        self.object_map.insert(ob_var_name, ob_id);
        return self;
    }

    pub fn get_ev<'a, S: AsRef<str>>(
        &self,
        ev_var_name: S,
        ocel: &'a LinkedOCEL,
    ) -> Option<&'a OCELEvent> {
        match self.event_map.get(ev_var_name.as_ref()) {
            Some(e_id) => ocel.event_map.get(e_id).copied(),
            None => None,
        }
    }

    pub fn get_ob<'a, S: AsRef<str>>(
        &self,
        ob_var_name: S,
        ocel: &'a LinkedOCEL,
    ) -> Option<&'a OCELObject> {
        match self.object_map.get(ob_var_name.as_ref()) {
            Some(o_id) => ocel.object_map.get(o_id).copied(),
            None => None,
        }
    }
}

impl BindingBox {
    pub fn expand(self: &Self, ocel: &LinkedOCEL) -> Vec<Binding> {
        let mut ret = vec![Binding::default()];

        for step in BindingStep::get_binding_order(self) {
            match &step {
                BindingStep::BindEv(ev_var_name) => {
                    let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                    ret = ret
                        .into_iter()
                        .flat_map(|b| {
                            ev_types
                                .iter()
                                .flat_map(|ev_type| ocel.events_of_type.get(ev_type).unwrap())
                                .map(move |e| {
                                    b.clone().expand_with_ev(ev_var_name.clone(), e.id.clone())
                                })
                        })
                        .collect();
                }
                BindingStep::BindOb(ob_var_name) => {
                    let ob_types = self.new_object_vars.get(ob_var_name).unwrap();
                    ret = ret
                        .into_iter()
                        .flat_map(|b| {
                            ob_types
                                .iter()
                                .flat_map(|ob_type| ocel.objects_of_type.get(ob_type).unwrap())
                                .map(move |o| {
                                    b.clone().expand_with_ob(ob_var_name.clone(), o.id.clone())
                                })
                        })
                        .collect();
                }
                BindingStep::BindObFromEv(ob_var_name, from_ev_var_name) => {
                    ret = ret
                        .into_iter()
                        .flat_map(|b| {
                            let e = b.get_ev(from_ev_var_name, ocel).unwrap();
                            let obj_types = self.new_object_vars.get(ob_var_name).unwrap();
                            e.relationships
                                .iter()
                                .flat_map(|rs| rs)
                                .filter(|rel| {
                                    obj_types.contains(
                                        &ocel.object_map.get(&rel.object_id).unwrap().object_type,
                                    )
                                })
                                .map(move |rel| {
                                    b.clone()
                                        .expand_with_ob(ob_var_name.clone(), rel.object_id.clone())
                                })
                        })
                        .collect();
                }
                BindingStep::BindObFromOb(ob_var_name, from_ob_var_name) => todo!(),
                BindingStep::BindEvFromOb(ev_var_name, from_ob_var_name) => todo!(),
                BindingStep::Filter(f) => todo!(),
            }
        }

        ret
    }
}
/// Maps a variable name to a set of object/event types
///
/// The type indicates the type of the value the object/event variable should be bound to
pub type NewVariables = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone)]
pub enum FilterConstraint {
    /// Object is associated with event
    ObjectAssociatedWithEvent(String, String),
}

#[derive(Debug, Clone)]
pub enum BindingStep {
    BindEv(String),
    BindOb(String),
    BindObFromEv(String, String),
    BindObFromOb(String, String),
    BindEvFromOb(String, String),
    Filter(FilterConstraint),
}

impl BindingStep {
    /// Get a binding order from a binding box
    ///
    /// A binding order has the following properties
    ///
    /// * All object/event variables that the input binding box binds are bound before they are used in a filter
    /// * The order should enable fast construction, i.e., it should create as few unnecessary bindings in between as possible
    ///
    /// For that, it e.g., could make sense to first bind an event variable and then use the bound event to bind object variables
    pub fn get_binding_order(bbox: &BindingBox) -> Vec<Self> {
        let mut ret = Vec::new();

        // bool: is object variable
        let mut var_requiring_bindings: HashSet<(bool, String)> = bbox
            .new_event_vars
            .iter()
            .map(|(v_name, _)| (false, v_name.clone()))
            .chain(
                bbox.new_object_vars
                    .iter()
                    .map(|(v_name, _)| (true, v_name.clone())),
            )
            .collect();
        // Maps a variable A to the set of variable that can be bound based on A
        let mut var_can_bind: HashMap<(bool, String), HashSet<(bool, String)>> = HashMap::new();

        // First count how many other variables depend on a variable (gather them in a set)
        for f in &bbox.filter_constraint {
            match f {
                FilterConstraint::ObjectAssociatedWithEvent(ob_var, ev_var) => {
                    var_can_bind
                        .entry((true, ob_var.clone()))
                        .or_default()
                        .insert((false, ev_var.clone()));
                    var_can_bind
                        .entry((false, ev_var.clone()))
                        .or_default()
                        .insert((true, ob_var.clone()));
                }
            }
        }
        let mut expansion = var_can_bind
            .clone()
            .into_iter()
            .sorted_by_key(|(_, v)| (v.len() as i32))
            .map(|(k, _)| k)
            .collect_vec();
        while !expansion.is_empty() {
            if let Some((is_obj_var, var_name)) = expansion.pop() {
                let can_bind_vars = var_can_bind.get(&(is_obj_var, var_name.clone())).unwrap();
                // Remove variable that are bound by current var-name...
                expansion = expansion
                    .into_iter()
                    .filter(|e| !can_bind_vars.contains(e))
                    .collect();
                // ...and then add them to be considered next
                expansion.extend(
                    can_bind_vars
                        .iter()
                        .filter(|x| var_requiring_bindings.contains(*x))
                        .cloned(),
                );
                if is_obj_var {
                    if var_requiring_bindings.contains(&(is_obj_var, var_name.clone())) {
                        ret.push(BindingStep::BindOb(var_name.clone()));
                        var_requiring_bindings.remove(&(is_obj_var, var_name.clone()));
                    }
                    for (is_child_obj_var, child_var_name) in can_bind_vars {
                        if var_requiring_bindings
                            .contains(&(*is_child_obj_var, child_var_name.clone()))
                        {
                            var_requiring_bindings
                                .remove(&(*is_child_obj_var, child_var_name.clone()));
                            if *is_child_obj_var {
                                ret.push(BindingStep::BindObFromOb(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                ))
                            } else {
                                ret.push(BindingStep::BindEvFromOb(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                ))
                            }
                        }
                    }
                } else {
                    if var_requiring_bindings.contains(&(is_obj_var, var_name.clone())) {
                        ret.push(BindingStep::BindEv(var_name.clone()));
                        var_requiring_bindings.remove(&(is_obj_var, var_name.clone()));
                    }
                    for (is_child_obj_var, child_var_name) in can_bind_vars {
                        if var_requiring_bindings
                            .contains(&(*is_child_obj_var, child_var_name.clone()))
                        {
                            var_requiring_bindings
                                .remove(&(*is_child_obj_var, child_var_name.clone()));
                            if *is_child_obj_var {
                                ret.push(BindingStep::BindObFromEv(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                ))
                            } else {
                                eprintln!("Can't bind an Event based on another Event");
                            }
                        }
                    }
                }
            }
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use process_mining::{import_ocel_json_from_path, import_ocel_xml, import_ocel_xml_file};

    use crate::preprocessing::preprocess::link_ocel_info;

    use super::{BindingBox, BindingStep, FilterConstraint, NewVariables};

    #[test]
    fn basic_binding_box() {
        let binding_box = BindingBox {
            new_event_vars: vec![(
                "ev_0".to_string(),
                vec!["pay order".to_string()].into_iter().collect(),
            )]
            .into_iter()
            .collect(),
            new_object_vars: vec![
                (
                    "or_0".to_string(),
                    vec!["orders".to_string()].into_iter().collect(),
                ),
                // (
                //     "or_1".to_string(),
                //     vec!["items".to_string()].into_iter().collect(),
                // ),
            ]
            .into_iter()
            .collect(),
            filter_constraint: vec![
                FilterConstraint::ObjectAssociatedWithEvent("or_0".to_string(), "ev_0".to_string()),
                // FilterConstraint::ObjectAssociatedWithEvent("or_1".to_string(), "ev_0".to_string()),
            ],
        };

        let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
        let linked_ocel = link_ocel_info(&ocel);
        let steps = BindingStep::get_binding_order(&binding_box);
        println!("Steps: {:?}", steps);
        let res = binding_box.expand(&linked_ocel);
        println!("Output binding size: {}", res.len())
    }
}
