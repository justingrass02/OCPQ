use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;
use process_mining::ocel::ocel_struct::{OCELEvent, OCELObject};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::preprocessing::preprocess::{
    get_events_of_type_associated_with_objects, EventIndex, LinkedOCEL, ObjectIndex,
};

pub struct BindingBox {
    pub new_event_vars: NewEventVariables,
    pub new_object_vars: NewObjectVariables,
    pub filter_constraint: Vec<FilterConstraint>,
}

#[derive(Debug, Default, Clone)]
pub struct Binding {
    pub event_map: HashMap<EventVariable, EventIndex>,
    pub object_map: HashMap<ObjectVariable, ObjectIndex>,
}

impl Binding {
    pub fn expand_with_ev(mut self, ev_var: EventVariable, ev_index: EventIndex) -> Self {
        self.event_map.insert(ev_var, ev_index);
        self
    }
    pub fn expand_with_ob(mut self, ev_var: ObjectVariable, ob_index: ObjectIndex) -> Self {
        self.object_map.insert(ev_var, ob_index);
        self
    }
    pub fn get_ev<'a>(
        &self,
        ev_var: &EventVariable,
        ocel: &'a LinkedOCEL,
    ) -> Option<&'a OCELEvent> {
        match self.event_map.get(ev_var) {
            Some(ev_index) => ocel.ev_by_index(ev_index),
            None => None,
        }
    }
    pub fn get_ob<'a>(
        &self,
        ob_var: &ObjectVariable,
        ocel: &'a LinkedOCEL,
    ) -> Option<&'a OCELObject> {
        match self.object_map.get(ob_var) {
            Some(ob_index) => ocel.ob_by_index(ob_index),
            None => None,
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binding [\n")?;
        write!(f, "\tEvents: {{ ")?;
        for (i, (ev_var, ev_index)) in self.event_map.iter().enumerate() {
            write!(f, "{} => {}", ev_var, ev_index)?;
            if i < self.event_map.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, " }}\n\tObjects: {{ ")?;
        for (i, (ob_var, ob_index)) in self.object_map.iter().enumerate() {
            write!(f, "{} => {}", ob_var, ob_index)?;
            if i < self.object_map.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, " }}")?;
        write!(f, "\n]")?;
        Ok(())
    }
}

impl BindingBox {
    pub fn expand_empty(&self, ocel: &LinkedOCEL) -> Vec<Binding> {
        self.expand(vec![Binding::default()], ocel)
    }

    pub fn expand_with_steps_empty(
        &self,
        ocel: &LinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        self.expand_with_steps(vec![Binding::default()], ocel, steps)
    }

    pub fn expand(&self, parent_bindings: Vec<Binding>, ocel: &LinkedOCEL) -> Vec<Binding> {
        self.expand_with_steps(parent_bindings, ocel, &BindingStep::get_binding_order(self))
    }

    pub fn expand_with_steps(
        &self,
        parent_bindings: Vec<Binding>,
        ocel: &LinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        let mut ret = parent_bindings;

        for step in steps {
            match &step {
                BindingStep::BindEv(ev_var, time_constr) => {
                    let ev_types = self.new_event_vars.get(ev_var).unwrap();
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            ev_types
                                .iter()
                                .flat_map(|ev_type| ocel.events_of_type_index.get(ev_type).unwrap())
                                .filter_map(move |e_index| {
                                    let e = ocel.ev_by_index(e_index).unwrap();
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
                                        Some(b.clone().expand_with_ev(*ev_var, *e_index))
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect();
                }
                BindingStep::BindOb(ob_var) => {
                    let ob_types = self.new_object_vars.get(ob_var).unwrap();
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            ob_types
                                .iter()
                                .flat_map(|ob_type| {
                                    ocel.objects_of_type_index.get(ob_type).unwrap()
                                })
                                .map(move |o_index| b.clone().expand_with_ob(*ob_var, *o_index))
                        })
                        .collect();
                }
                BindingStep::BindObFromEv(ob_var, from_ev_var, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let e = b.get_ev(from_ev_var, ocel).unwrap();
                            let obj_types = self.new_object_vars.get(ob_var).unwrap();
                            e.relationships
                                .iter()
                                .flatten()
                                .filter(|rel| {
                                    obj_types.contains(
                                        &ocel.object_map.get(&rel.object_id).unwrap().object_type,
                                    ) && (qualifier.is_none()
                                        || qualifier.as_ref().unwrap() == &rel.qualifier)
                                })
                                .map(move |rel| {
                                    b.clone().expand_with_ob(
                                        *ob_var,
                                        *ocel.object_index_map.get(&rel.object_id).unwrap(),
                                    )
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
                                    b.clone().expand_with_ob(
                                        *ob_var_name,
                                        *ocel.object_index_map.get(&rel.object_id).unwrap(),
                                    )
                                })
                        })
                        .collect()
                }
                BindingStep::BindEvFromOb(ev_var_name, from_ob_var_name, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let ob = b.get_ob(from_ob_var_name, ocel).unwrap().clone();
                            let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                            get_events_of_type_associated_with_objects(
                                ocel,
                                &crate::constraints::EventType::AnyOf {
                                    values: ev_types.iter().cloned().collect(),
                                },
                                vec![ob.id.clone()],
                            )
                            .into_iter()
                            .filter_map(move |e| {
                                if qualifier.is_none()
                                    || e.relationships.as_ref().is_some_and(|rels| {
                                        rels.iter().any(|rel| {
                                            rel.object_id == ob.id
                                                && &rel.qualifier == qualifier.as_ref().unwrap()
                                        })
                                    })
                                {
                                    Some(b.clone().expand_with_ev(
                                        *ev_var_name,
                                        *ocel.event_index_map.get(&e.id).unwrap(),
                                    ))
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
                                let ob = b.get_ob(obj_var, ocel).unwrap();
                                let ev = b.get_ev(ev_var, ocel).unwrap();
                                ev.relationships.as_ref().is_some_and(|rels| {
                                    rels.iter().any(|rel| {
                                        rel.object_id == ob.id
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
                                let ob2 = b.get_ob(obj_var_2, ocel).unwrap();
                                ob1.relationships.as_ref().is_some_and(|rels| {
                                    rels.iter().any(|rel| {
                                        rel.object_id == ob2.id
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
pub type NewObjectVariables = HashMap<ObjectVariable, HashSet<String>>;
pub type NewEventVariables = HashMap<EventVariable, HashSet<String>>;

#[derive(Debug, Clone)]
pub enum FilterConstraint {
    /// Object is associated with event (optionally through a qualifier)
    ObjectAssociatedWithEvent(ObjectVariable, EventVariable, Qualifier),
    /// Object1 is associated with object2 (optionally through a qualifier)
    ObjectAssociatedWithObject(ObjectVariable, ObjectVariable, Qualifier),
    /// Time duration betweeen event1 and event2 is in the specified interval (min,max) (given in Some(seconds); where None represents no restriction)
    TimeBetweenEvents(EventVariable, EventVariable, (Option<f64>, Option<f64>)),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Variable {
    Event(EventVariable),
    Object(ObjectVariable),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EventVariable(usize);
impl From<usize> for EventVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for EventVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ev_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectVariable(usize);
impl From<usize> for ObjectVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for ObjectVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ob_{}", self.0)
    }
}

pub type Qualifier = Option<String>;

type DurationIntervalSeconds = (Option<f64>, Option<f64>);

#[derive(Debug, Clone)]
pub enum BindingStep {
    BindEv(
        EventVariable,
        Option<Vec<(EventVariable, DurationIntervalSeconds)>>,
    ),
    BindOb(ObjectVariable),
    /// Bind ob
    BindObFromEv(ObjectVariable, EventVariable, Qualifier),
    BindObFromOb(ObjectVariable, ObjectVariable, Qualifier),
    BindEvFromOb(EventVariable, ObjectVariable, Qualifier),
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

        let mut var_requiring_bindings: HashSet<Variable> = bbox
            .new_event_vars
            .keys()
            .map(|v| Variable::Event(*v))
            .chain(bbox.new_object_vars.keys().map(|v| Variable::Object(*v)))
            .collect();


        let new_vars = var_requiring_bindings.clone();
        // Maps a variable A to the set of variable that can be bound based on A
        let mut var_can_bind: HashMap<Variable, HashSet<Variable>> = HashMap::new();
        // Additional info, with a qualifier and the index of a filter constraint
        let mut var_can_bind_with_qualifier: HashMap<
            Variable,
            HashSet<(Variable, Qualifier, usize)>,
        > = HashMap::new();
        for ev_var in bbox.new_event_vars.keys() {
            var_can_bind.insert(Variable::Event(*ev_var), HashSet::new());
            var_can_bind_with_qualifier.insert(Variable::Event(*ev_var), HashSet::new());
        }
        for ob_var in bbox.new_object_vars.keys() {
            var_can_bind.insert(Variable::Object(*ob_var), HashSet::new());
            var_can_bind_with_qualifier.insert(Variable::Object(*ob_var), HashSet::new());
        }

        // First count how many other variables depend on a variable (gather them in a set)
        for (i, f) in bbox.filter_constraint.iter().enumerate() {
            match f {
                FilterConstraint::ObjectAssociatedWithEvent(ob_var, ev_var, qualifier) => {
                    var_can_bind
                        .entry(Variable::Object(*ob_var))
                        .or_default()
                        .insert(Variable::Event(*ev_var));
                    var_can_bind_with_qualifier
                        .entry(Variable::Object(*ob_var))
                        .or_default()
                        .insert((Variable::Event(*ev_var), qualifier.clone(), i));

                    var_can_bind
                        .entry(Variable::Event(*ev_var))
                        .or_default()
                        .insert(Variable::Object(*ob_var));
                    var_can_bind_with_qualifier
                        .entry(Variable::Event(*ev_var))
                        .or_default()
                        .insert((Variable::Object(*ob_var), qualifier.clone(), i));
                }
                FilterConstraint::ObjectAssociatedWithObject(ob_var_1, ob_var_2, qualifier) => {
                    var_can_bind
                        .entry(Variable::Object(*ob_var_1))
                        .or_default()
                        .insert(Variable::Object(*ob_var_2));
                    var_can_bind_with_qualifier
                        .entry(Variable::Object(*ob_var_1))
                        .or_default()
                        .insert((Variable::Object(*ob_var_2), qualifier.clone(), i));
                }
                _ => {}
            }
        }
        let mut filter_indices_incoporated = HashSet::new();
        let mut expansion = var_can_bind
            .clone()
            .into_iter()
            // Prefer binding events over objects first
            // Prefer already bound events + objects
            .sorted_by_key(|(v, vs)| {
              (if new_vars.contains(v) { 0 } else { 10000000 } +  (vs.len() as i32) * 1000 + if let Variable::Object(_) = v { 0 } else { 1 })
            })
            .map(|(k, _)| k)
            .collect_vec();
        
        while !expansion.is_empty() {
            if let Some(var) = expansion.pop() {
                let can_bind_vars = var_can_bind.get(&var).unwrap();
                let can_bind_vars_qualified = var_can_bind_with_qualifier.get(&var).unwrap();
                // Remove variable that are bound by current var-name...
                expansion.retain(|e| !can_bind_vars.contains(e));
                // ...and then add them to be considered next
                expansion.extend(
                    can_bind_vars
                        .iter()
                        .filter(|x| var_requiring_bindings.contains(*x))
                        .cloned(),
                );
                match var {
                    Variable::Object(ob_var) => {
                        if var_requiring_bindings.contains(&var) {
                            ret.push(BindingStep::BindOb(ob_var));
                            var_requiring_bindings.remove(&var);
                        }
                        for (child_var, qualifier, f_index) in can_bind_vars_qualified {
                            if var_requiring_bindings.contains(child_var) {
                                filter_indices_incoporated.insert(*f_index);
                                var_requiring_bindings.remove(child_var);
                                match child_var {
                                    Variable::Object(child_ob_var) => {
                                        ret.push(BindingStep::BindObFromOb(
                                            *child_ob_var,
                                            ob_var,
                                            qualifier.clone(),
                                        ))
                                    }
                                    Variable::Event(child_ev_var) => {
                                        ret.push(BindingStep::BindEvFromOb(
                                            *child_ev_var,
                                            ob_var,
                                            qualifier.clone(),
                                        ))
                                    }
                                }
                            }
                        }
                    }
                    Variable::Event(ev_var) => {
                        if var_requiring_bindings.contains(&var) {
                            // Try to get a time filter by looking for filter constraints which impose a restriction in relation to an already bound event variable
                            // Also, if successfull, emit the underlying time filter from the list (as it is already covered)
                            let time_filter = bbox
                                .filter_constraint
                                .iter()
                                .enumerate()
                                .filter_map(|(index, c)| match c {
                                    FilterConstraint::TimeBetweenEvents(
                                        ev_var_0,
                                        ev_var_1,
                                        time,
                                    ) => {
                                        if ev_var_1 == &ev_var
                                            && !var_requiring_bindings
                                                .contains(&Variable::Event(*ev_var_0))
                                        {
                                            // ...also mark the time filter as already incoporated (it holds automatically, if the incoporated into the event binding step)
                                            filter_indices_incoporated.insert(index);
                                            return Some((*ev_var_0, *time));
                                        } else if ev_var_0 == &ev_var
                                            && !var_requiring_bindings
                                                .contains(&Variable::Event(*ev_var_1))
                                        {
                                            // ...also mark the time filter as already incoporated (it holds automatically, if the incoporated into the event binding step)
                                            filter_indices_incoporated.insert(index);
                                            // In this case, the time constraint is set in the other direction
                                            // But we can invert the time constraint to get a valid time filter for ev_var_0
                                            return Some((
                                                *ev_var_1,
                                                (time.1.map(|s| -s), time.0.map(|s| -s)),
                                            ));
                                        }
                                        None
                                    }
                                    _ => None,
                                })
                                .collect_vec();
                            ret.push(BindingStep::BindEv(
                                ev_var,
                                if time_filter.is_empty() {
                                    None
                                } else {
                                    Some(time_filter)
                                },
                            ));
                            var_requiring_bindings.remove(&var);
                        }
                        for (child_var, qualifier, f_index) in can_bind_vars_qualified {
                            if var_requiring_bindings.contains(child_var) {
                                filter_indices_incoporated.insert(*f_index);
                                var_requiring_bindings.remove(child_var);
                                if let Variable::Object(child_ob_var) = child_var {
                                    ret.push(BindingStep::BindObFromEv(
                                        *child_ob_var,
                                        ev_var,
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
                    0.into(),
                    vec!["place order".to_string()].into_iter().collect(),
                ),
                (
                    1.into(),
                    vec!["place order".to_string()].into_iter().collect(),
                ),
            ]
            .into_iter()
            .collect(),
            new_object_vars: vec![
                (
                    0.into(),
                    vec!["customers".to_string()].into_iter().collect(),
                ),
                (1.into(), vec!["orders".to_string()].into_iter().collect()),
                (2.into(), vec!["orders".to_string()].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
            filter_constraint: vec![
                FilterConstraint::TimeBetweenEvents(0.into(), 1.into(), (Some(0.0000000001), None)),
                FilterConstraint::ObjectAssociatedWithEvent(1.into(), 0.into(), None),
                FilterConstraint::ObjectAssociatedWithEvent(2.into(), 1.into(), None),
                FilterConstraint::ObjectAssociatedWithObject(
                    0.into(),
                    1.into(),
                    Some("places".to_string()),
                ),
                FilterConstraint::ObjectAssociatedWithObject(
                    0.into(),
                    2.into(),
                    Some("places".to_string()),
                ),
            ],
        };

        let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
        let linked_ocel = link_ocel_info(&ocel);
        let steps = BindingStep::get_binding_order(&binding_box);
        println!("Steps: {:?}", steps);
        let now = Instant::now();
        let res = binding_box.expand_with_steps_empty(&linked_ocel, &steps);
        println!("Output binding size: {} in {:?}", res.len(), now.elapsed());
        println!("First binding: {}", res.first().unwrap());
    }


    #[test]
    fn connected_binding_box() {
        let binding_box = BindingBox {
            new_event_vars: vec![
            ]
            .into_iter()
            .collect(),
            new_object_vars: vec![
                (0.into(), vec!["orders".to_string()].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
            filter_constraint: vec![
            ],
        };
        let binding_box_2 = BindingBox {
            new_event_vars: vec![
                (0.into(),vec!["pay order".to_string()].into_iter().collect())
            ]
            .into_iter()
            .collect(),
            new_object_vars: vec![
                // (1.into(),vec!["employees".to_string()].into_iter().collect())
            ]
            .into_iter()
            .collect(),
            filter_constraint: vec![
                FilterConstraint::ObjectAssociatedWithEvent(0.into(), 0.into(), None),
                // FilterConstraint::ObjectAssociatedWithEvent(1.into(), 0.into(), None)
            ],
        };

        let ocel = import_ocel_json_from_path("../data/order-management.json").unwrap();
        let linked_ocel = link_ocel_info(&ocel);
        let now = Instant::now();
        println!("Steps: {:?}", BindingStep::get_binding_order(&binding_box));
        let res = binding_box.expand_empty(&linked_ocel);
        println!("Output binding size: {} in {:?}", res.len(), now.elapsed());
        println!("Steps: {:?}", BindingStep::get_binding_order(&binding_box_2));
        let (min,max) = (1,1);
        let mut outcome = Vec::new();
        for b in res {
            let res2 = binding_box_2.expand(vec![b.clone()],&linked_ocel);
            if res2.len() >= min && res2.len() <= max {
                outcome.push((b,true));
            }else{
                outcome.push((b,false));
            }
        }
        let num_sat = outcome.iter().filter(|(_,sat)| *sat).count() as f32;
        println!("Total time {:?} {}", now.elapsed(), num_sat / outcome.len() as f32);
    }
}
