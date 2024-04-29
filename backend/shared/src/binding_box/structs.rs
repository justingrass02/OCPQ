use core::time;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use process_mining::ocel::ocel_struct::{OCELEvent, OCELObject};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::preprocessing::preprocess::{get_events_of_type_associated_with_objects, LinkedOCEL};

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

    pub fn get_ev_id<'a, S: AsRef<str>>(&'a self, ev_var_name: S) -> Option<&'a String> {
        self.event_map.get(ev_var_name.as_ref())
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

    pub fn get_ob_id<'a, S: AsRef<str>>(&'a self, ob_var_name: S) -> Option<&'a String> {
        self.object_map.get(ob_var_name.as_ref())
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
        self.expand_with_steps(ocel, &BindingStep::get_binding_order(self))
    }

    pub fn expand_with_steps(
        self: &Self,
        ocel: &LinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        let mut ret = vec![Binding::default()];

        for step in steps {
            match &step {
                BindingStep::BindEv(ev_var_name, time_constr) => {
                    let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            ev_types
                                .iter()
                                .flat_map(|ev_type| ocel.events_of_type.get(ev_type).unwrap())
                                .filter_map(move |e| {
                                    if time_constr.is_none()
                                        || time_constr.as_ref().unwrap().iter().all(
                                            |(ref_ev_var_name, (min_sec, max_sec))| {
                                                let ref_ev =
                                                    b.get_ev(ref_ev_var_name, ocel).unwrap();
                                                let duration_diff = (e.time - ref_ev.time)
                                                    .num_milliseconds()
                                                    as f64
                                                    / 1000.0;
                                                !min_sec
                                                    .is_some_and(|min_sec| duration_diff < min_sec)
                                                    && !max_sec.is_some_and(|max_sec| {
                                                        duration_diff > max_sec
                                                    })
                                            },
                                        )
                                    {
                                        Some(
                                            b.clone()
                                                .expand_with_ev(ev_var_name.clone(), e.id.clone()),
                                        )
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect();
                }
                BindingStep::BindOb(ob_var_name) => {
                    let ob_types = self.new_object_vars.get(ob_var_name).unwrap();
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            ob_types
                                .iter()
                                .flat_map(|ob_type| ocel.objects_of_type.get(ob_type).unwrap())
                                .map(move |o| {
                                    b.clone().expand_with_ob(ob_var_name.clone(), o.id.clone())
                                })
                        })
                        .collect();
                }
                BindingStep::BindObFromEv(ob_var_name, from_ev_var_name, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let e = b.get_ev(from_ev_var_name, ocel).unwrap();
                            let obj_types = self.new_object_vars.get(ob_var_name).unwrap();
                            e.relationships
                                .iter()
                                .flat_map(|rs| rs)
                                .filter(|rel| {
                                    obj_types.contains(
                                        &ocel.object_map.get(&rel.object_id).unwrap().object_type,
                                    ) && (qualifier.is_none()
                                        || qualifier.as_ref().unwrap() == &rel.qualifier)
                                })
                                .map(move |rel| {
                                    b.clone()
                                        .expand_with_ob(ob_var_name.clone(), rel.object_id.clone())
                                })
                        })
                        .collect();
                }
                BindingStep::BindObFromOb(ob_var_name, from_ob_var_name, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let ob = b.get_ob(from_ob_var_name, ocel).unwrap();
                            ob.relationships
                                .as_ref()
                                .into_iter()
                                .flatten()
                                .filter(|rel| {
                                    qualifier.is_none()
                                        || &rel.qualifier == qualifier.as_ref().unwrap()
                                })
                                .map(move |rel| {
                                    b.clone()
                                        .expand_with_ob(ob_var_name.clone(), rel.object_id.clone())
                                })
                        })
                        .collect()
                }
                BindingStep::BindEvFromOb(ev_var_name, from_ob_var_name, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let ob_id = b.get_ob_id(from_ob_var_name).unwrap().clone();
                            let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                            get_events_of_type_associated_with_objects(
                                ocel,
                                &crate::constraints::EventType::AnyOf {
                                    values: ev_types.iter().cloned().collect(),
                                },
                                vec![ob_id.clone()],
                            )
                            .into_iter()
                            .filter_map(move |e| {
                                if qualifier.is_none()
                                    || e.relationships.as_ref().is_some_and(|rels| {
                                        rels.iter().any(|rel| {
                                            rel.object_id == ob_id
                                                && &rel.qualifier == qualifier.as_ref().unwrap()
                                        })
                                    })
                                {
                                    Some(
                                        b.clone().expand_with_ev(ev_var_name.clone(), e.id.clone()),
                                    )
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();
                }
                BindingStep::Filter(f) => {
                    ret = ret
                        .into_par_iter()
                        .filter(|b| match f {
                            FilterConstraint::ObjectAssociatedWithEvent(
                                obj_var,
                                ev_var,
                                qualifier,
                            ) => {
                                let ob_id = b.get_ob_id(obj_var).unwrap();
                                let ev = b.get_ev(ev_var, ocel).unwrap();
                                ev.relationships.as_ref().is_some_and(|rels| {
                                    rels.iter().any(|rel| {
                                        &rel.object_id == ob_id
                                            && if let Some(q) = qualifier {
                                                &rel.qualifier == q
                                            } else {
                                                true
                                            }
                                    })
                                })
                            }
                            FilterConstraint::ObjectAssociatedWithObject(
                                obj_var_1,
                                obj_var_2,
                                qualifier,
                            ) => {
                                let ob1 = b.get_ob(obj_var_1, ocel).unwrap();
                                let ob2_id = b.get_ob_id(obj_var_2).unwrap();
                                ob1.relationships.as_ref().is_some_and(|rels| {
                                    rels.iter().any(|rel| {
                                        &rel.object_id == ob2_id
                                            && if let Some(q) = qualifier {
                                                &rel.qualifier == q
                                            } else {
                                                true
                                            }
                                    })
                                })
                            }
                            FilterConstraint::TimeBetweenEvents(
                                ev_var_1,
                                ev_var_2,
                                (min_sec, max_sec),
                            ) => {
                                let e1 = b.get_ev(ev_var_1, ocel).unwrap();
                                let e2 = b.get_ev(ev_var_2, ocel).unwrap();
                                let duration_diff =
                                    (e2.time - e1.time).num_milliseconds() as f64 / 1000.0;
                                !min_sec.is_some_and(|min_sec| duration_diff < min_sec)
                                    && !max_sec.is_some_and(|max_sec| duration_diff > max_sec)
                            }
                        })
                        .collect()
                }
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
    /// Object is associated with event (optionally through a qualifier)
    ObjectAssociatedWithEvent(String, String, Option<String>),
    /// Object1 is associated with object2 (optionally through a qualifier)
    ObjectAssociatedWithObject(String, String, Option<String>),
    /// Time duration betweeen event1 and event2 is in the specified interval (min,max) (given in Some(seconds); where None represents no restriction)
    TimeBetweenEvents(String, String, (Option<f64>, Option<f64>)),
}

#[derive(Debug, Clone)]
pub enum BindingStep {
    BindEv(String, Option<Vec<(String, (Option<f64>, Option<f64>))>>),
    BindOb(String),
    /// Bind ob
    BindObFromEv(String, String, Option<String>),
    BindObFromOb(String, String, Option<String>),
    BindEvFromOb(String, String, Option<String>),
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
        // Additional info, with a qualifier and the index of a filter constraint
        let mut var_can_bind_with_qualifier: HashMap<
            (bool, String),
            HashSet<((bool, String), Option<String>, usize)>,
        > = HashMap::new();
        for (ev_var_name, _) in &bbox.new_event_vars {
            var_can_bind.insert((false, ev_var_name.clone()), HashSet::new());
            var_can_bind_with_qualifier.insert((false, ev_var_name.clone()), HashSet::new());
        }
        for (ob_var_name, _) in &bbox.new_object_vars {
            var_can_bind.insert((true, ob_var_name.clone()), HashSet::new());
            var_can_bind_with_qualifier.insert((true, ob_var_name.clone()), HashSet::new());
        }

        // First count how many other variables depend on a variable (gather them in a set)
        for (i, f) in bbox.filter_constraint.iter().enumerate() {
            match f {
                FilterConstraint::ObjectAssociatedWithEvent(ob_var, ev_var, qualifier) => {
                    var_can_bind
                        .get_mut(&(true, ob_var.clone()))
                        .unwrap()
                        .insert((false, ev_var.clone()));
                    var_can_bind_with_qualifier
                        .get_mut(&(true, ob_var.clone()))
                        .unwrap()
                        .insert(((false, ev_var.clone()), qualifier.clone(), i));

                    var_can_bind
                        .get_mut(&(false, ev_var.clone()))
                        .unwrap()
                        .insert((true, ob_var.clone()));
                    var_can_bind_with_qualifier
                        .get_mut(&(false, ev_var.clone()))
                        .unwrap()
                        .insert(((true, ob_var.clone()), qualifier.clone(), i));
                }
                FilterConstraint::ObjectAssociatedWithObject(ob_var_1, ob_var_2, qualifier) => {
                    var_can_bind
                        .get_mut(&(true, ob_var_1.clone()))
                        .unwrap()
                        .insert((true, ob_var_2.clone()));
                    var_can_bind_with_qualifier
                        .get_mut(&(true, ob_var_1.clone()))
                        .unwrap()
                        .insert(((true, ob_var_2.clone()), qualifier.clone(), i));
                }
                _ => {}
            }
        }
        let mut filter_indices_incoporated = HashSet::new();
        let mut expansion = var_can_bind
            .clone()
            .into_iter()
            // Prefer binding events over objects first
            .sorted_by_key(|((is_obj, _), v)| (v.len() as i32) * 1000 + if *is_obj { 0 } else { 1 })
            .map(|(k, _)| k)
            .collect_vec();
        while !expansion.is_empty() {
            if let Some((is_obj_var, var_name)) = expansion.pop() {
                let can_bind_vars = var_can_bind.get(&(is_obj_var, var_name.clone())).unwrap();
                let can_bind_vars_qualified = var_can_bind_with_qualifier
                    .get(&(is_obj_var, var_name.clone()))
                    .unwrap();
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
                    for ((is_child_obj_var, child_var_name), qualifier, f_index) in
                        can_bind_vars_qualified
                    {
                        if var_requiring_bindings
                            .contains(&(*is_child_obj_var, child_var_name.clone()))
                        {
                            filter_indices_incoporated.insert(f_index);
                            var_requiring_bindings
                                .remove(&(*is_child_obj_var, child_var_name.clone()));
                            if *is_child_obj_var {
                                ret.push(BindingStep::BindObFromOb(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                    qualifier.clone(),
                                ))
                            } else {
                                ret.push(BindingStep::BindEvFromOb(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                    qualifier.clone(),
                                ))
                            }
                        }
                    }
                } else {
                    if var_requiring_bindings.contains(&(is_obj_var, var_name.clone())) {
                        // Try to get a time filter by looking for filter constraints which impose a restriction in relation to an already bound event variable
                        // In theory, if successfull, we could also remove the filter constraints from the step list, but we might want to allow for some inaccuracies (i.e., false positives) in this stage
                        let time_filter = bbox
                            .filter_constraint
                            .iter()
                            .filter_map(|c| match c {
                                FilterConstraint::TimeBetweenEvents(ev_var_0, ev_var_1, time) => {
                                    if ev_var_1 == &var_name
                                        && !var_requiring_bindings
                                            .contains(&(false, ev_var_0.clone()))
                                    {
                                        return Some((ev_var_0.clone(), time.clone()));
                                    } else if ev_var_0 == &var_name
                                        && !var_requiring_bindings
                                            .contains(&(false, ev_var_1.clone()))
                                    {
                                        // In this case, the time constraint is set in the other direction
                                        // But we can invert the time constraint to get a valid time filter for ev_var_0
                                        return Some((
                                            ev_var_1.clone(),
                                            (time.1.map(|s| -s), time.0.map(|s| -s)),
                                        ));
                                    }
                                    None
                                }
                                _ => None,
                            })
                            .collect_vec();
                        ret.push(BindingStep::BindEv(
                            var_name.clone(),
                            if time_filter.is_empty() {
                                None
                            } else {
                                Some(time_filter)
                            },
                        ));
                        var_requiring_bindings.remove(&(is_obj_var, var_name.clone()));
                    }
                    for ((is_child_obj_var, child_var_name), qualifier, f_index) in
                        can_bind_vars_qualified
                    {
                        if var_requiring_bindings
                            .contains(&(*is_child_obj_var, child_var_name.clone()))
                        {
                            filter_indices_incoporated.insert(f_index);
                            var_requiring_bindings
                                .remove(&(*is_child_obj_var, child_var_name.clone()));
                            if *is_child_obj_var {
                                ret.push(BindingStep::BindObFromEv(
                                    child_var_name.clone(),
                                    var_name.clone(),
                                    qualifier.clone(),
                                ))
                            } else {
                                eprintln!("Can't bind an Event based on another Event");
                            }
                        }
                    }
                }
            }
        }
        ret.extend(
            bbox.filter_constraint
                .iter()
                .enumerate()
                .filter(|(i, _)| !filter_indices_incoporated.contains(i))
                .map(|(_, f)| BindingStep::Filter(f.clone())),
        );
        ret
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use process_mining::import_ocel_json_from_path;

    use crate::{
        binding_box::structs::FilterConstraint, preprocessing::preprocess::link_ocel_info,
    };

    use super::{BindingBox, BindingStep};

    #[test]
    fn basic_binding_box() {
        let binding_box = BindingBox {
            new_event_vars: vec![
                (
                    "ev_0".to_string(),
                    vec!["place order".to_string()].into_iter().collect(),
                ),
                (
                    "ev_1".to_string(),
                    vec!["place order".to_string()].into_iter().collect(),
                ),
            ]
            .into_iter()
            .collect(),
            new_object_vars: vec![
                (
                    "or_0".to_string(),
                    vec!["orders".to_string()].into_iter().collect(),
                ),
                (
                    "or_1".to_string(),
                    vec!["orders".to_string()].into_iter().collect(),
                ),
                // (
                //     "it_0".to_string(),
                //     vec!["items".to_string()].into_iter().collect(),
                // ),
                // (
                //     "it_1".to_string(),
                //     vec!["items".to_string()].into_iter().collect(),
                // ),
            ]
            .into_iter()
            .collect(),
            filter_constraint: vec![
                FilterConstraint::TimeBetweenEvents(
                    "ev_0".to_string(),
                    "ev_1".to_string(),
                    (Some(0.0000000001), None),
                ),
                FilterConstraint::ObjectAssociatedWithEvent(
                    "or_0".to_string(),
                    "ev_0".to_string(),
                    None,
                ),
                FilterConstraint::ObjectAssociatedWithEvent(
                    "or_1".to_string(),
                    "ev_1".to_string(),
                    None,
                ),
                // FilterConstraint::ObjectAssociatedWithEvent(
                //     "or_0".to_string(),
                //     "ev_0".to_string(),
                //     None,
                // ),
                // FilterConstraint::ObjectAssociatedWithEvent(
                //     "it_0".to_string(),
                //     "ev_0".to_string(),
                //     None,
                // ),
                // FilterConstraint::ObjectAssociatedWithObject(
                //     "or_0".to_string(),
                //     "it_0".to_string(),
                //     // None
                //     Some("comprises".to_string()),
                // ),
            ],
        };

        let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
        let linked_ocel = link_ocel_info(&ocel);
        let steps = BindingStep::get_binding_order(&binding_box);
        println!("Steps: {:?}", steps);
        let now = Instant::now();
        let res = binding_box.expand_with_steps(&linked_ocel, &steps);
        println!("Output binding size: {} in {:?}", res.len(), now.elapsed());
        println!("First binding: {:#?}",res.first());
    }
}
