use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use itertools::Itertools;

use super::structs::{BindingBox, BindingStep, Filter, Qualifier, Variable};

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
        let mut bound_vars: HashSet<_> = bbox
            .filters
            .iter()
            .flat_map(|f| f.get_involved_variables())
            .filter(|var| !new_vars.contains(var))
            .collect();
        // Maps a variable A to the set of variable that can be bound based on A
        let mut var_can_bind: HashMap<Variable, HashSet<Variable>> = bound_vars
            .iter()
            .map(|v| (v.clone(), HashSet::new()))
            .collect();
        // Additional info, with a qualifier and the index of a filter constraint
        let mut var_can_bind_with_qualifier: HashMap<
            Variable,
            HashSet<(Variable, Qualifier, usize,bool)>,
        > = bound_vars
        .iter()
        .map(|v| (v.clone(), HashSet::new()))
        .collect();
        for ev_var in bbox.new_event_vars.keys() {
            var_can_bind.insert(Variable::Event(*ev_var), HashSet::new());
            var_can_bind_with_qualifier.insert(Variable::Event(*ev_var), HashSet::new());
        }
        for ob_var in bbox.new_object_vars.keys() {
            var_can_bind.insert(Variable::Object(*ob_var), HashSet::new());
            var_can_bind_with_qualifier.insert(Variable::Object(*ob_var), HashSet::new());
        }

        // First count how many other variables depend on a variable (gather them in a set)
        for (i, f) in bbox.filters.iter().enumerate() {
            match f {
                Filter::O2E {
                    object,
                    event,
                    qualifier,
                } => {
                    var_can_bind
                        .entry(Variable::Object(*object))
                        .or_default()
                        .insert(Variable::Event(*event));
                    var_can_bind_with_qualifier
                        .entry(Variable::Object(*object))
                        .or_default()
                        .insert((Variable::Event(*event), qualifier.clone(), i,false));

                    var_can_bind
                        .entry(Variable::Event(*event))
                        .or_default()
                        .insert(Variable::Object(*object));
                    var_can_bind_with_qualifier
                        .entry(Variable::Event(*event))
                        .or_default()
                        .insert((Variable::Object(*object), qualifier.clone(), i,true));
                }
                Filter::O2O {
                    object,
                    other_object,
                    qualifier,
                } => {
                    var_can_bind
                        .entry(Variable::Object(*object))
                        .or_default()
                        .insert(Variable::Object(*other_object));
                    var_can_bind_with_qualifier
                        .entry(Variable::Object(*object))
                        .or_default()
                        .insert((Variable::Object(*other_object), qualifier.clone(), i,false));

                        var_can_bind
                        .entry(Variable::Object(*other_object))
                        .or_default()
                        .insert(Variable::Object(*object));
                    var_can_bind_with_qualifier
                        .entry(Variable::Object(*other_object))
                        .or_default()
                        .insert((Variable::Object(*object), qualifier.clone(), i,true));
                }
                _ => {}
            }
        }
        let mut filter_indices_incoporated = HashSet::new();

        fn add_supported_filters(
            bbox: &BindingBox,
            filter_indices_incoporated: &mut HashSet<usize>,
            var_requiring_bindings: &mut HashSet<Variable>,
            ret: &mut Vec<BindingStep>,
        ) {
            bbox.filters
                .iter()
                .enumerate()
                .filter(|(index, filter_constraint)| {
                    if !filter_indices_incoporated.contains(index) {
                        var_requiring_bindings
                            .intersection(&filter_constraint.get_involved_variables())
                            .count()
                            == 0
                    } else {
                        false
                    }
                })
                .collect_vec()
                .into_iter()
                .for_each(|(index, filter_constraint)| {
                    ret.push(BindingStep::Filter(filter_constraint.clone()));
                    filter_indices_incoporated.insert(index);
                });
        }

        let mut expansion = var_can_bind
            .clone()
            .into_iter().filter(|(v,_vs)| !bound_vars.contains(v))
            // Prefer binding events over objects first
            .sorted_by_key(|(v, vs)| {
                    let can_be_bound = bound_vars
                        .iter()
                        .any(|bv| var_can_bind.get(bv).unwrap().contains(v));
                    (vs.len() as i32) * 10 + if can_be_bound {100} else {0} 
                    + if let Variable::Object(_) = v { 0 } else { 1 }
            })
            .map(|(k, _)| k)
            .collect_vec();
        while !expansion.is_empty() {
            if let Some(var) = expansion.pop() {
                if bound_vars.contains(&var) {
                    continue;
                }
                if let Some((v,(_var, qualifier, filter_index,reversed))) = bound_vars
                    .iter()
                    .flat_map(|v| {
                        var_can_bind_with_qualifier
                            .get(&v)
                            .unwrap()
                            .iter()
                            .filter(|(x, _q, _filter_index,_reversed)| x == &var)
                            .next().map(|t| (v,t))
                    })
                    .next()
                {
                    // `var` can be bound based on `v`!
                    filter_indices_incoporated.insert(*filter_index);
                    match v {
                        Variable::Event(v_ev) => match var {
                            Variable::Object(var_ob) => ret.push(BindingStep::BindObFromEv(
                                var_ob,
                                *v_ev,
                                qualifier.clone(),
                            )),
                            _ => {
                                eprintln!("Can not bind an event based on another event.")
                            }
                        },
                        Variable::Object(v_ob) => match var {
                            Variable::Event(var_ev) => ret.push(BindingStep::BindEvFromOb(
                                var_ev,
                                *v_ob,
                                qualifier.clone(),
                            )),
                            Variable::Object(var_ob) => ret.push(BindingStep::BindObFromOb(
                                var_ob,
                                *v_ob,
                                qualifier.clone(),
                                *reversed
                            )),
                        },
                    }
                } else {
                    match var {
                        Variable::Event(var_ev) => ret.push(BindingStep::BindEv(var_ev, None)),
                        Variable::Object(var_ob) => ret.push(BindingStep::BindOb(var_ob)),
                    }
                }
                var_requiring_bindings.remove(&var);
                bound_vars.insert(var);
                add_supported_filters(
                    bbox,
                    &mut filter_indices_incoporated,
                    &mut var_requiring_bindings,
                    &mut ret,
                );
            }
            expansion.sort_by_key(|var| {
                let can_be_bound = bound_vars
                    .iter()
                    .any(|bv| var_can_bind.get(bv).unwrap().contains(var));
                if can_be_bound {
                    100
                } else {
                    0
                }
            })
        }
        // while !expansion.is_empty() {
        //     println!("Expansion? {:?}",expansion);
        //     if let Some(var) = expansion.pop() {
        //         println!("Var {:?}",var);
        //         let can_bind_vars = var_can_bind.get(&var).unwrap();
        //         let can_bind_vars_qualified = var_can_bind_with_qualifier.get(&var).unwrap();
        //         // Remove variable that are bound by current var-name...
        //         expansion.retain(|e| !can_bind_vars.contains(e));
        //         // ...and then add them to be considered next
        //         expansion.extend(
        //             can_bind_vars
        //                 .iter()
        //                 .filter(|x| var_requiring_bindings.contains(*x))
        //                 .cloned(),
        //         );
        //         match var {
        //             Variable::Object(ob_var) => {
        //                 if var_requiring_bindings.contains(&var) {
        //                     ret.push(BindingStep::BindOb(ob_var));
        //                     var_requiring_bindings.remove(&var);
        //                 }
        //                 for (child_var, qualifier, f_index) in can_bind_vars_qualified
        //                     .iter()
        //                     .sorted_by(|(c_1, _, _), (_c_2, _, _)| {
        //                         if matches!(c_1, Variable::Event(_)) {
        //                             Ordering::Less
        //                         } else {
        //                             Ordering::Greater
        //                         }
        //                     })
        //                 {
        //                     if var_requiring_bindings.contains(child_var) {
        //                         filter_indices_incoporated.insert(*f_index);
        //                         var_requiring_bindings.remove(child_var);
        //                         match child_var {
        //                             Variable::Object(child_ob_var) => {
        //                                 ret.push(BindingStep::BindObFromOb(
        //                                     *child_ob_var,
        //                                     ob_var,
        //                                     qualifier.clone(),
        //                                 ))
        //                             }
        //                             Variable::Event(child_ev_var) => {
        //                                 ret.push(BindingStep::BindEvFromOb(
        //                                     *child_ev_var,
        //                                     ob_var,
        //                                     qualifier.clone(),
        //                                 ))
        //                             }
        //                         }
        //                         add_supported_filters(
        //                             bbox,
        //                             &mut filter_indices_incoporated,
        //                             &mut var_requiring_bindings,
        //                             &mut ret,
        //                         );
        //                     }
        //                 }
        //             }
        //             Variable::Event(ev_var) => {
        //                 if var_requiring_bindings.contains(&var) {
        //                     // Try to get a time filter by looking for filter constraints which impose a restriction in relation to an already bound event variable
        //                     // Also, if successfull, emit the underlying time filter from the list (as it is already covered)
        //                     let time_filter = bbox
        //                         .filters
        //                         .iter()
        //                         .enumerate()
        //                         .filter_map(|(index, c)| match c {
        //                             Filter::TimeBetweenEvents {
        //                                 from_event: ev_var_0,
        //                                 to_event: ev_var_1,
        //                                 min_seconds,
        //                                 max_seconds,
        //                             } => {
        //                                 if ev_var_1 == &ev_var
        //                                     && !var_requiring_bindings
        //                                         .contains(&Variable::Event(*ev_var_0))
        //                                 {
        //                                     // ...also mark the time filter as already incoporated (it holds automatically, if the incoporated into the event binding step)
        //                                     filter_indices_incoporated.insert(index);
        //                                     return Some((
        //                                         *ev_var_0,
        //                                         (*min_seconds, *max_seconds),
        //                                     ));
        //                                 } else if ev_var_0 == &ev_var
        //                                     && !var_requiring_bindings
        //                                         .contains(&Variable::Event(*ev_var_1))
        //                                 {
        //                                     // ...also mark the time filter as already incoporated (it holds automatically, if the incoporated into the event binding step)
        //                                     filter_indices_incoporated.insert(index);
        //                                     // In this case, the time constraint is set in the other direction
        //                                     // But we can invert the time constraint to get a valid time filter for ev_var_0
        //                                     return Some((
        //                                         *ev_var_1,
        //                                         (max_seconds.map(|s| -s), min_seconds.map(|s| -s)),
        //                                     ));
        //                                 }
        //                                 None
        //                             }
        //                             _ => None,
        //                         })
        //                         .collect_vec();
        //                     ret.push(BindingStep::BindEv(
        //                         ev_var,
        //                         if time_filter.is_empty() {
        //                             None
        //                         } else {
        //                             Some(time_filter)
        //                         },
        //                     ));
        //                     var_requiring_bindings.remove(&var);
        //                     add_supported_filters(
        //                         bbox,
        //                         &mut filter_indices_incoporated,
        //                         &mut var_requiring_bindings,
        //                         &mut ret,
        //                     );
        //                 }
        //                 for (child_var, qualifier, f_index) in can_bind_vars_qualified {
        //                     if var_requiring_bindings.contains(child_var) {
        //                         filter_indices_incoporated.insert(*f_index);
        //                         var_requiring_bindings.remove(child_var);
        //                         if let Variable::Object(child_ob_var) = child_var {
        //                             ret.push(BindingStep::BindObFromEv(
        //                                 *child_ob_var,
        //                                 ev_var,
        //                                 qualifier.clone(),
        //                             ))
        //                         } else {
        //                             eprintln!("Can't bind an Event based on another Event");
        //                         }

        //                         add_supported_filters(
        //                             bbox,
        //                             &mut filter_indices_incoporated,
        //                             &mut var_requiring_bindings,
        //                             &mut ret,
        //                         );
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     add_supported_filters(
        //         bbox,
        //         &mut filter_indices_incoporated,
        //         &mut var_requiring_bindings,
        //         &mut ret,
        //     );
        // }

        ret.extend(
            bbox.filters
                .iter()
                .enumerate()
                .filter(|(i, _)| !filter_indices_incoporated.contains(i))
                .map(|(_, f)| BindingStep::Filter(f.clone())),
        );
        // println!("{:?}",ret);
        ret
    }
}
