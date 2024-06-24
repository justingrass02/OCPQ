use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use process_mining::ocel::ocel_struct::{OCELEvent, OCELObject};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use ts_rs::TS;

use crate::preprocessing::linked_ocel::{EventIndex, IndexLinkedOCEL, ObjectIndex};
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variable {
    Event(EventVariable),
    Object(ObjectVariable),
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventVariable(pub usize);
impl From<usize> for EventVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectVariable(pub usize);
impl From<usize> for ObjectVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub type Qualifier = Option<String>;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        ocel: &'a IndexLinkedOCEL,
    ) -> Option<&'a OCELEvent> {
        match self.event_map.get(ev_var) {
            Some(ev_index) => ocel.ev_by_index(ev_index),
            None => None,
        }
    }
    pub fn get_ob<'a>(
        &self,
        ob_var: &ObjectVariable,
        ocel: &'a IndexLinkedOCEL,
    ) -> Option<&'a OCELObject> {
        match self.object_map.get(ob_var) {
            Some(ob_index) => ocel.ob_by_index(ob_index),
            None => None,
        }
    }

    pub fn get_ev_index(&self, ev_var: &EventVariable) -> Option<&EventIndex> {
        self.event_map.get(ev_var)
    }
    pub fn get_ob_index(&self, ob_var: &ObjectVariable) -> Option<&ObjectIndex> {
        self.object_map.get(ob_var)
    }
}

/// Maps a variable name to a set of object/event types
///
/// The value set indicates the types of the value the object/event variable should be bound to
pub type NewObjectVariables = HashMap<ObjectVariable, HashSet<String>>;
pub type NewEventVariables = HashMap<EventVariable, HashSet<String>>;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingBox {
    pub new_event_vars: NewEventVariables,
    pub new_object_vars: NewObjectVariables,
    pub filters: Vec<Filter>,
    pub size_filters: Vec<SizeFilter>,
    pub constraints: Vec<Constraint>,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingBoxTree {
    pub nodes: Vec<BindingBoxTreeNode>,
    #[serde_as(as = "Vec<(_, _)>")]
    #[ts(as = "Vec<((usize, usize), String)>")]
    pub edge_names: HashMap<(usize, usize), String>
    // #[serde_as(as = "Vec<(_, _)>")]
    // #[ts(as = "Vec<((usize, usize), (Option<usize>, Option<usize>))>")]
    // pub size_constraints: HashMap<(usize, usize), (Option<usize>, Option<usize>)>,
}

impl BindingBoxTree {
    pub fn evaluate(&self, ocel: &IndexLinkedOCEL) -> EvaluationResults {
        if let Some(root) = self.nodes.first() {
            let (mut ret, violation) = root.evaluate(0, 0, Binding::default(), self, ocel);
            // ret.push((0, Binding::default(), violation));
            ret
        } else {
            vec![]
        }
    }

    pub fn get_ev_vars(&self) -> HashSet<EventVariable> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                BindingBoxTreeNode::Box(b, _) => Some(b.new_event_vars.keys()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect()
    }

    pub fn get_ob_vars(&self) -> HashSet<ObjectVariable> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                BindingBoxTreeNode::Box(b, _) => Some(b.new_object_vars.keys()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect()
    }
}
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingBoxTreeNode {
    Box(BindingBox, Vec<usize>),
    OR(usize, usize),
    AND(usize, usize),
    NOT(usize),
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ViolationReason {
    TooFewMatchingEvents(usize),
    TooManyMatchingEvents(usize),
    NoChildrenOfORSatisfied,
    LeftChildOfANDUnsatisfied,
    RightChildOfANDUnsatisfied,
    BothChildrenOfANDUnsatisfied,
    ChildrenOfNOTSatisfied,
    ChildNotSatisfied,
    ConstraintNotSatisfied(usize),
}

pub type EvaluationResult = (usize, Binding, Option<ViolationReason>);
pub type EvaluationResults = Vec<EvaluationResult>;
use rayon::prelude::*;

impl BindingBoxTreeNode {
    pub fn evaluate(
        &self,
        own_index: usize,
        parent_index: usize,
        parent_binding: Binding,
        tree: &BindingBoxTree,
        ocel: &IndexLinkedOCEL,
    ) -> (EvaluationResults, Vec<(Binding,Option<ViolationReason>)>) {
        match self {
            BindingBoxTreeNode::Box(bbox, children) => {
                let expanded: Vec<Binding> = bbox.expand(vec![parent_binding.clone()], ocel);
                enum BindingResult {
                    FilteredOutBySizeFilter,
                    Sat(Binding,EvaluationResults),
                    Viol(Binding, ViolationReason, EvaluationResults),
                }
                let re: Vec<_> = expanded
                    .into_par_iter()
                    .map(|b| {
                        let mut passed_size_filter = true;
                        let mut all_res: EvaluationResults = Vec::new();
                        let mut child_res: HashMap<String, Vec<(Binding,Option<ViolationReason>)>> =
                            HashMap::new();
                        // let mut child_res = Vec::with_capacity(children.len());
                        for c in children {
                            let c_name = tree.edge_names.get(&(own_index,*c)).cloned().unwrap_or(format!("NO NAME PROVIDED - {c}"));
                            let (mut c_res, violations) =
                        // Evaluate Child
                            tree.nodes[*c].evaluate(*c, own_index, b.clone(), tree, ocel);
                            // Check if size binding count is passes size filters
                            passed_size_filter =
                                bbox.size_filters
                                    .iter()
                                    .all(|size_filter| match size_filter {
                                        SizeFilter::NumChilds {
                                            child_name,
                                            min,
                                            max,
                                        } => {
                                            // println!("{child_index} {c} Min: {:?} Max: {:?} Len: {}",min,max,violations.len());
                                            if child_name != &c_name {
                                                true
                                            } else {
                                                if min.is_some_and(|min| violations.len() < min) {
                                                    false
                                                } else if max
                                                    .is_some_and(|max| violations.len() > max)
                                                {
                                                    false
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    });
                            if !passed_size_filter {
                                // println!("Did not pass size filter");
                                return BindingResult::FilteredOutBySizeFilter;
                            }
                            child_res.insert(c_name, violations);
                            all_res.extend(c_res);
                        }
                        for (constr_index,constr) in bbox.constraints.iter().enumerate() {
                            let viol = match constr {
                                Constraint::Filter { filter } => {
                                    if filter.check_binding(&b, ocel) {
                                        None
                                    }else {
                                        Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                                    }
                                },
                                Constraint::SizeFilter { filter } => match filter {
                                    SizeFilter::NumChilds {
                                        child_name,
                                        min,
                                        max,
                                    } => {
                                        println!("Children: {:?}; Child Name: {:?} Min: {:?} Max: {:?}", children,child_name, min, max);
                                        if let Some(len) =
                                        child_res.get(child_name).map(|r| r.len())
                                        {
                                            if min.is_some_and(|min| len < min) {
                                                Some(ViolationReason::TooFewMatchingEvents(len))
                                            } else if max.is_some_and(|max| len > max) {
                                                Some(ViolationReason::TooManyMatchingEvents(len))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                },
                                Constraint::SAT { child_names } => {
                                    let violated = child_names.iter().all(|child_name| {
                                        if let Some(c_res) = child_res.get(child_name) {
                                            if c_res.iter().any(|(b,v)| v.is_some()) {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    });
                                    if violated {
                                        Some(ViolationReason::ChildNotSatisfied)
                                    } else {
                                        None
                                    }
                                }
                                Constraint::NOT { child_names } => todo!(),
                                Constraint::OR { child_names } => {
                                    println!("Child indices: {:?}, Children: {:?}", child_names, children);
                                    let any_sat = child_names.iter().any(|child_name| {
                                        if let Some(c_res) = child_res.get(child_name) {
                                            if c_res.iter().all(|(b,v)| v.is_none()) {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    });
                                    if any_sat {
                                        None
                                    }else {
                                        Some(ViolationReason::NoChildrenOfORSatisfied)
                                    }
                                },
                                Constraint::AND { child_names } => todo!(),
                            };
                            if let Some(vr) = viol {
                                all_res.push((own_index, b.clone(), Some(vr)));
                                return BindingResult::Viol(b,vr, all_res);
                            }
                        }
                        all_res.push((own_index, b.clone(), None));
                        return BindingResult::Sat(b,all_res);
                    })
                    .collect();

                let res = re
                    .into_par_iter()
                    .fold(
                        || (EvaluationResults::new(), Vec::new()),
                        |(mut a, mut b), x| match x {
                            BindingResult::FilteredOutBySizeFilter => (a, b),
                            BindingResult::Sat(binding,r) => {
                                a.extend(r);
                                b.push((binding,None));
                                (a, b)
                            }
                            BindingResult::Viol(binding,v, r) => {
                                a.extend(r);
                                b.push((binding,Some(v)));
                                (a, b)
                            }
                        },
                    )
                    .reduce(
                        || (EvaluationResults::new(), Vec::new()),
                        |(mut a, mut b), (x, y)| {
                            a.extend(x);
                            b.extend(y);
                            (a, b)
                        },
                    );

                res

                // let (passed_size_filter, sat, ret) = expanded
                //     .into_par_iter()
                //     .flat_map_iter(|b| {
                //         let mut passed_size_filter = true;
                //         children.iter().map(move |c| {
                //             let (mut c_res, violation) =
                //                 tree.nodes[*c].evaluate(*c, own_index, b.clone(), tree, ocel);
                //             c_res.push((*c, b.clone(), violation));
                //             passed_size_filter = if let Some(_x) = violation {
                //                 (true, c_res)
                //             } else {
                //                 (false, c_res)
                //             }
                //         })
                //     })
                //     .reduce(
                //         || (false, vec![]),
                //         |(violated1, res1), (violated2, res2)| {
                //             (
                //                 violated1 || violated2,
                //                 res1.iter().chain(res2.iter()).cloned().collect(),
                //             )
                //         },
                //     );

                // if vio.is_none() && sat {
                //     vio = Some(ViolationReason::ChildNotSatisfied)
                // }
                // (ret, vio)
            }
            // BindingBoxTreeNode::OR(i1, i2) => {
            //     let node1 = &tree.nodes[*i1];
            //     let node2 = &tree.nodes[*i2];

            //     let mut ret = vec![];

            //     let (res_1, violation_1) =
            //         node1.evaluate(*i1, own_index, parent_binding.clone(), tree, ocel);

            //     ret.extend(res_1);
            //     ret.push((*i1, parent_binding.clone(), violation_1));

            //     let (res_2, violation_2) =
            //         node2.evaluate(*i2, own_index, parent_binding.clone(), tree, ocel);

            //     ret.extend(res_2);
            //     ret.push((*i2, parent_binding.clone(), violation_2));

            //     if violation_1.is_some() && violation_2.is_some() {
            //         return (ret, Some(ViolationReason::NoChildrenOfORSatisfied));
            //     }
            //     (ret, None)
            // }
            // BindingBoxTreeNode::AND(i1, i2) => {
            //     let node1 = &tree.nodes[*i1];
            //     let node2 = &tree.nodes[*i2];

            //     let mut ret = vec![];

            //     let (res_1, violation_1) =
            //         node1.evaluate(*i1, own_index, parent_binding.clone(), tree, ocel);

            //     ret.push((*i1, parent_binding.clone(), violation_1));
            //     ret.extend(res_1);
            //     let (res_2, violation_2) =
            //         node2.evaluate(*i2, own_index, parent_binding.clone(), tree, ocel);
            //     ret.push((*i2, parent_binding.clone(), violation_2));
            //     ret.extend(res_2);

            //     if violation_1.is_some() {
            //         return (ret, Some(ViolationReason::LeftChildOfANDUnsatisfied));
            //     } else if violation_2.is_some() {
            //         return (ret, Some(ViolationReason::RightChildOfANDUnsatisfied));
            //     }
            //     (ret, None)
            // }
            // BindingBoxTreeNode::NOT(i) => {
            //     let mut ret = vec![];
            //     let node = &tree.nodes[*i];

            //     let (res_c, violation_c) =
            //         node.evaluate(*i, own_index, parent_binding.clone(), tree, ocel);
            //     ret.extend(res_c);
            //     ret.push((*i, parent_binding.clone(), violation_c));
            //     if violation_c.is_some() {
            //         // NOT satisfied
            //         (ret, None)
            //     } else {
            //         (ret, Some(ViolationReason::ChildrenOfNOTSatisfied))
            //     }
            // }
            _ => todo!(),
        }
    }
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Filter {
    /// Object is associated with event (optionally through a qualifier)
    O2E {
        object: ObjectVariable,
        event: EventVariable,
        qualifier: Qualifier,
    },
    /// Object1 is associated with object2 (optionally through a qualifier)
    O2O {
        object: ObjectVariable,
        other_object: ObjectVariable,
        qualifier: Qualifier,
    },
    /// Time duration betweeen event1 and event2 is in the specified interval (min,max) (given in Some(seconds); where None represents no restriction)
    TimeBetweenEvents {
        from_event: EventVariable,
        to_event: EventVariable,
        min_seconds: Option<f64>,
        max_seconds: Option<f64>,
    },
}

impl Filter {
    pub fn check_binding(&self, b: &Binding, ocel: &IndexLinkedOCEL) -> bool {
        match self {
            Filter::O2E {
                object,
                event,
                qualifier,
            } => {
                let ob = b.get_ob(object, ocel).unwrap();
                let ev = b.get_ev(event, ocel).unwrap();
                ev.relationships.as_ref().is_some_and(
                    |rels: &Vec<process_mining::ocel::ocel_struct::OCELRelationship>| {
                        rels.iter().any(|rel| {
                            rel.object_id == ob.id
                                && if let Some(q) = qualifier {
                                    &rel.qualifier == q
                                } else {
                                    true
                                }
                        })
                    },
                )
            }
            Filter::O2O {
                object,
                other_object,
                qualifier,
            } => {
                let ob1 = b.get_ob(object, ocel).unwrap();
                let ob2 = b.get_ob(other_object, ocel).unwrap();
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
            Filter::TimeBetweenEvents {
                from_event: ev_var_1,
                to_event: ev_var_2,
                min_seconds: min_sec,
                max_seconds: max_sec,
            } => {
                let e1 = b.get_ev(ev_var_1, ocel).unwrap();
                let e2 = b.get_ev(ev_var_2, ocel).unwrap();
                let duration_diff = (e2.time - e1.time).num_milliseconds() as f64 / 1000.0;
                !min_sec.is_some_and(|min_sec| duration_diff < min_sec)
                    && !max_sec.is_some_and(|max_sec| duration_diff > max_sec)
            }
        }
    }
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SizeFilter {
    // The nth child should be between (min,max) interval, where None represent no bound
    NumChilds {
        child_name: NodeEdgeName,
        min: Option<usize>,
        max: Option<usize>,
    },
}

type NodeEdgeName = String;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Constraint {
    Filter { filter: Filter },
    SizeFilter { filter: SizeFilter },
    SAT { child_names: Vec<NodeEdgeName> },
    NOT { child_names: Vec<NodeEdgeName> },
    OR { child_names: Vec<NodeEdgeName> },
    AND { child_names: Vec<NodeEdgeName> },
}

impl Filter {
    pub fn get_involved_variables(&self) -> HashSet<Variable> {
        match self {
            Filter::O2E {
                object,
                event,
                qualifier,
            } => vec![Variable::Object(*object), Variable::Event(*event)]
                .into_iter()
                .collect(),
            Filter::O2O {
                object,
                other_object,
                qualifier,
            } => vec![Variable::Object(*object), Variable::Object(*other_object)]
                .into_iter()
                .collect(),
            Filter::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds,
                max_seconds,
            } => vec![Variable::Event(*from_event), Variable::Event(*to_event)]
                .into_iter()
                .collect(),
            // Filter::ObjectAssociatedWithEvent(ov, ev, _) => {
            // }
            // Filter::ObjectAssociatedWithOtherObject(ov1, ov2, _) => {
            //     vec![Variable::Object(*ov1), Variable::Object(*ov2)]
            //         .into_iter()
            //         .collect()
            // }
            // Filter::TimeBetweenEvents(ev1, ev2, _) => {
            //     vec![Variable::Event(*ev1), Variable::Event(*ev2)]
            //         .into_iter()
            //         .collect()
            // }
        }
    }
}

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
    Filter(Filter),
}

//
// Display Implementations
//

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Binding [")?;
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

impl Display for ObjectVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ob_{}", self.0)
    }
}

impl Display for EventVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ev_{}", self.0)
    }
}
