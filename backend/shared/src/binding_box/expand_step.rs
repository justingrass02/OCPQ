use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::preprocessing::linked_ocel::{EventOrObjectIndex, IndexLinkedOCEL};

use super::structs::{Binding, BindingBox, BindingStep};

/// This can slightly reduce memory usage by filtering out unfitting bindings before collecting into a vec
/// However, the filters may be checked multiple times
#[inline(always)]
fn check_next_filters(
    b: Binding,
    next_step: usize,
    steps: &[BindingStep],
    ocel: &IndexLinkedOCEL,
) -> Option<Binding> {
    for step in steps.iter().skip(next_step) {
        if let BindingStep::Filter(f) = &step {
            if f.check_binding(&b, ocel) {
                continue;
            } else {
                return None;
            }
        } else {
            break;
        }
    }
    Some(b)
}

impl BindingBox {
    pub fn expand_empty(&self, ocel: &IndexLinkedOCEL) -> Vec<Binding> {
        self.expand(Binding::default(), ocel)
    }

    pub fn expand_with_steps_empty(
        &self,
        ocel: &IndexLinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        self.expand_with_steps(Binding::default(), ocel, steps)
    }

    pub fn expand(&self, parent_binding: Binding, ocel: &IndexLinkedOCEL) -> Vec<Binding> {
        let order = BindingStep::get_binding_order(self,Some(&parent_binding), Some(&ocel));
        self.expand_with_steps(parent_binding, ocel, &order)
    }

    pub fn expand_with_steps(
        &self,
        parent_binding: Binding,
        ocel: &IndexLinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        let mut ret = vec![parent_binding];
        let mut sizes_per_step: Vec<usize> = Vec::with_capacity(steps.len());
        for step_index in 0..steps.len() {
            let step = &steps[step_index];
            match &step {
                BindingStep::BindEv(ev_var, time_constr) => {
                    let ev_types = self.new_event_vars.get(ev_var).unwrap();
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            ev_types
                                .iter()
                                .flat_map(|ev_type| ocel.events_of_type.get(ev_type).unwrap())
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
                                        check_next_filters(
                                            b.clone().expand_with_ev(*ev_var, *e_index),
                                            step_index + 1,
                                            steps,
                                            ocel,
                                        )
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
                                .flat_map(|ob_type| ocel.objects_of_type.get(ob_type).unwrap())
                                .filter_map(move |o_index| {
                                    check_next_filters(
                                        b.clone().expand_with_ob(*ob_var, *o_index),
                                        step_index + 1,
                                        steps,
                                        ocel,
                                    )
                                })
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
                                        &ocel.ob_by_id(&rel.object_id).unwrap().object_type,
                                    ) && (qualifier.is_none()
                                        || qualifier.as_ref().unwrap() == &rel.qualifier)
                                })
                                .filter_map(move |rel| {
                                    check_next_filters(
                                        b.clone().expand_with_ob(
                                            *ob_var,
                                            *ocel.object_index_map.get(&rel.object_id).unwrap(),
                                        ),
                                        step_index + 1,
                                        steps,
                                        ocel,
                                    )
                                })
                        })
                        .collect();
                }
                BindingStep::BindObFromOb(ob_var_name, from_ob_var_name, qualifier, reversed) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let ob_index = b.get_ob_index(from_ob_var_name).unwrap();
                            ocel.get_symmetric_rels_ob(ob_index)
                                .unwrap()
                                .iter()
                                .filter_map(move |(to_index, rev, qual)| {
                                    if let EventOrObjectIndex::Object(to_ob_index) = to_index {
                                        if rev == reversed
                                            && (qualifier.is_none()
                                                || qual == qualifier.as_ref().unwrap())
                                        {
                                            let allowed_types =
                                                self.new_object_vars.get(ob_var_name)?;
                                            let o = ocel.ob_by_index(to_ob_index)?;
                                            if allowed_types.contains(&o.object_type) {
                                                check_next_filters(
                                                    b.clone()
                                                        .expand_with_ob(*ob_var_name, *to_ob_index),
                                                    step_index + 1,
                                                    steps,
                                                    ocel,
                                                )
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect()
                }
                BindingStep::BindEvFromOb(ev_var_name, from_ob_var_name, qualifier) => {
                    ret = ret
                        .into_par_iter()
                        .flat_map_iter(|b| {
                            let ob_index = b.get_ob_index(from_ob_var_name).unwrap();
                            // let ob = ocel.ob_by_index(ob_index).unwrap();
                            let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                            ocel.get_symmetric_rels_ob(ob_index)
                                .unwrap()
                                .iter()
                                .filter_map(move |(rel_to, _reversed, q)| {
                                    if qualifier.is_none()
                                        || qualifier.as_ref().unwrap().contains(q)
                                    {
                                        if let EventOrObjectIndex::Event(rel_to_ev) = rel_to {
                                            let to_ev = ocel.ev_by_index(rel_to_ev)?;
                                            if ev_types.contains(&to_ev.event_type) {
                                                check_next_filters(
                                                    b.clone()
                                                        .expand_with_ev(*ev_var_name, *rel_to_ev),
                                                    step_index + 1,
                                                    steps,
                                                    ocel,
                                                )
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect();
                }
                // _ => {}
                BindingStep::Filter(f) => {
                    ret = ret
                        .into_par_iter()
                        .filter(|b| f.check_binding(b, ocel))
                        .collect()
                }
            }
            sizes_per_step.push(ret.len())
        }

        // if !steps.is_empty() {
        //     println!("Steps: {:?}", steps);
        //     println!("Set sizes: {:?}", sizes_per_step);
        // }
        ret
    }
}
