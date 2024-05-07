use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::preprocessing::linked_ocel::{get_events_of_type_associated_with_objects, IndexLinkedOCEL};

use super::structs::{Binding, BindingBox, BindingStep, FilterConstraint};

impl BindingBox {
    pub fn expand_empty(&self, ocel: &IndexLinkedOCEL) -> Vec<Binding> {
        self.expand(vec![Binding::default()], ocel)
    }

    pub fn expand_with_steps_empty(
        &self,
        ocel: &IndexLinkedOCEL,
        steps: &[BindingStep],
    ) -> Vec<Binding> {
        self.expand_with_steps(vec![Binding::default()], ocel, steps)
    }

    pub fn expand(&self, parent_bindings: Vec<Binding>, ocel: &IndexLinkedOCEL) -> Vec<Binding> {
        self.expand_with_steps(parent_bindings, ocel, &BindingStep::get_binding_order(self))
    }

    pub fn expand_with_steps(
        &self,
        parent_bindings: Vec<Binding>,
        ocel: &IndexLinkedOCEL,
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
                                    ocel.objects_of_type.get(ob_type).unwrap()
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
                                        &ocel.ob_by_id(&rel.object_id).unwrap().object_type,
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
                            let ob_index = b.get_ob_index(from_ob_var_name).unwrap();
                            let ob = ocel.ob_by_index(&ob_index).unwrap();
                            let ev_types = self.new_event_vars.get(ev_var_name).unwrap();
                            get_events_of_type_associated_with_objects(
                                ocel,
                                &crate::constraints::EventType::AnyOf {
                                    values: ev_types.iter().cloned().collect(),
                                },
                                vec![*ob_index],
                            )
                            .into_iter()
                            .filter_map(move |e_index| {
                                // TODO: Better to also have an relationship index in IndexLinkedOCEL
                                let e = ocel.ev_by_index(&e_index).unwrap();
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
