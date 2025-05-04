use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use itertools::Itertools;
use ordered_float::OrderedFloat;
use process_mining::ocel::ocel_struct::{OCELAttributeValue, OCELEvent, OCELObject};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use ts_rs::TS;

use crate::{
    cel::{add_cel_label, check_cel_predicate, get_vars_in_cel_program},
    preprocessing::linked_ocel::{EventIndex, EventOrObjectIndex, IndexLinkedOCEL, ObjectIndex},
};
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variable {
    Event(EventVariable),
    Object(ObjectVariable),
}

impl Variable {
    pub fn to_inner(&self) -> usize {
        match self {
            Variable::Event(ev) => ev.0,
            Variable::Object(ov) => ov.0,
        }
    }
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct EventVariable(pub usize);
impl From<usize> for EventVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ObjectVariable(pub usize);
impl From<usize> for ObjectVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub type Qualifier = Option<String>;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub event_map: BTreeMap<EventVariable, EventIndex>,
    pub object_map: BTreeMap<ObjectVariable, ObjectIndex>,
    pub label_map: BTreeMap<String, LabelValue>,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
pub enum LabelValue {
    String(std::sync::Arc<String>),
    Int(i64),
    Float(#[ts(as = "f64")] OrderedFloat<f64>),
    Bool(bool),
    Null,
}

impl LabelValue {
    pub fn to_string(&self) -> String {
        match self {
            LabelValue::String(arc) => arc.to_string(),
            LabelValue::Int(i) => i.to_string(),
            LabelValue::Float(f) => f.to_string(),
            LabelValue::Bool(b) => b.to_string(),
            LabelValue::Null => "null".to_string(),
        }
    }
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

    pub fn get_any_index(&self, var: &Variable) -> Option<EventOrObjectIndex> {
        match var {
            Variable::Event(ev) => self.get_ev_index(ev).map(|r| EventOrObjectIndex::Event(*r)),
            Variable::Object(ov) => self
                .get_ob_index(ov)
                .map(|r: &ObjectIndex| EventOrObjectIndex::Object(*r)),
        }
    }
}

/// Maps a variable name to a set of object/event types
///
/// The value set indicates the types of the value the object/event variable should be bound to
pub type NewObjectVariables = HashMap<ObjectVariable, HashSet<String>>;
pub type NewEventVariables = HashMap<EventVariable, HashSet<String>>;

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindingBox {
    pub new_event_vars: NewEventVariables,
    pub new_object_vars: NewObjectVariables,
    pub filters: Vec<Filter>,
    pub size_filters: Vec<SizeFilter>,
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    #[ts(optional)]
    #[ts(as = "Option<HashMap<EventVariable,FilterLabel>>")]
    pub ev_var_labels: HashMap<EventVariable, FilterLabel>,
    #[serde(default)]
    #[ts(optional)]
    #[ts(as = "Option<HashMap<EventVariable,FilterLabel>>")]
    pub ob_var_labels: HashMap<ObjectVariable, FilterLabel>,
    #[serde(default)]
    #[ts(optional)]
    #[ts(as = "Option<Vec<LabelFunction>>")]
    pub labels: Vec<LabelFunction>,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum FilterLabel {
    #[default]
    IGNORED,
    INCLUDED,
    EXCLUDED,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LabelFunction {
    pub label: String,
    pub cel: String,
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
    pub edge_names: HashMap<(usize, usize), String>, // #[serde_as(as = "Vec<(_, _)>")]
                                                     // #[ts(as = "Vec<((usize, usize), (Option<usize>, Option<usize>))>")]
                                                     // pub size_constraints: HashMap<(usize, usize), (Option<usize>, Option<usize>)>,
}

impl BindingBoxTree {
    pub fn evaluate(&self, ocel: &IndexLinkedOCEL) -> (EvaluationResults,bool) {
        if let Some(root) = self.nodes.first() {
            let ((ret, _violation),skipped) = root.evaluate(0, Binding::default(), self, ocel);
            // ret.push((0, Binding::default(), violation));
            (ret,skipped)
        } else {
            (vec![], false)
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
const UNNAMED: &str = "UNNAMED - ";
impl BindingBoxTreeNode {
    pub fn to_box(self) -> (BindingBox, Vec<usize>) {
        match self {
            BindingBoxTreeNode::Box(b, children) => (b, children),
            BindingBoxTreeNode::OR(c1, c2) => (
                BindingBox {
                    constraints: vec![Constraint::OR {
                        child_names: vec![
                            format!("{}{}", UNNAMED, c1),
                            format!("{}{}", UNNAMED, c2),
                        ],
                    }],
                    ..Default::default()
                },
                vec![c1, c2],
            ),
            BindingBoxTreeNode::AND(c1, c2) => (
                BindingBox {
                    constraints: vec![Constraint::AND {
                        child_names: vec![
                            format!("{}{}", UNNAMED, c1),
                            format!("{}{}", UNNAMED, c2),
                        ],
                    }],
                    ..Default::default()
                },
                vec![c1, c2],
            ),
            BindingBoxTreeNode::NOT(c1) => (
                BindingBox {
                    constraints: vec![Constraint::NOT {
                        child_names: vec![format!("{}{}", UNNAMED, c1)],
                    }],
                    ..Default::default()
                },
                vec![c1],
            ),
        }
    }

    pub fn as_box(&self) -> Option<&BindingBox> {
        match self {
            BindingBoxTreeNode::Box(b, _children) => Some(b),
            _ => None,
        }
    }
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
    UnknownChildSet,
}

pub type EvaluationResult = (usize, Binding, Option<ViolationReason>);
pub type EvaluationResults = Vec<EvaluationResult>;
use rayon::prelude::*;

impl BindingBoxTreeNode {
    pub fn evaluate(
        &self,
        own_index: usize,
        parent_binding: Binding,
        tree: &BindingBoxTree,
        ocel: &IndexLinkedOCEL,
    ) -> ((EvaluationResults, Vec<(Binding, Option<ViolationReason>)>),bool) {
        let (bbox, children) = match self.clone() {
            BindingBoxTreeNode::Box(b, cs) => (b, cs),
            x => x.to_box(),
        };
        // match self {
        //     BindingBoxTreeNode::Box(bbox, children) => {
        let (expanded, expanding_skipped_bindings): (Vec<Binding>,bool) = bbox.expand(parent_binding.clone(), ocel);
        enum BindingResult {
            FilteredOutBySizeFilter(Binding, EvaluationResults),
            Sat(Binding, EvaluationResults),
            Viol(Binding, ViolationReason, EvaluationResults),
        }
        let expanded_len = expanded.len();
        let it = rayon_cancel::CancelAdapter::new(expanded.into_par_iter());
        let x = it.canceller();
        let re: Vec<_> = it
            .map(|mut b| {
                let _passed_size_filter = true;
                // let mut all_res: EvaluationResults = Vec::new();
                // let mut child_res: HashMap<String, Vec<(Binding, Option<ViolationReason>)>> =
                //     HashMap::new();
                let mut all_res = Vec::new();
                let mut child_res = HashMap::with_capacity(children.len());
                for c in &children {
                    let c_name = tree
                        .edge_names
                        .get(&(own_index, *c))
                        .cloned()
                        .unwrap_or(format!("{UNNAMED}{c}"));
                    // c_name_map.insert(c_name.clone(), c);
                    let ((c_res, violations), _c_skipped) =
                        // Evaluate Child
                            tree.nodes[*c].evaluate(*c, b.clone(), tree, ocel);
                    child_res.insert(c_name, violations);
                    if children.len() * c_res.len() * expanded_len > 100_000_000 {
                        x.cancel();
                        println!(
                            "Too much too handle! {}*{}*{}={}",
                            child_res.len(),
                            c_res.len(),
                            expanded_len,
                            children.len() * c_res.len() * expanded_len
                        );
                    }

                    all_res.extend(c_res);
                }
                for label_fun in &bbox.labels {
                    add_cel_label(&mut b, Some(&child_res), ocel, label_fun);
                }
                for sf in &bbox.size_filters {
                    if !sf.check(&b, &child_res, ocel) {
                        // Vec::default to NOT include child results if a size filter filters the parent binding out
                        // Otherwise, pass all_res
                        return BindingResult::FilteredOutBySizeFilter(b.clone(), Vec::default());
                    }
                }

                for (constr_index, constr) in bbox.constraints.iter().enumerate() {
                    let viol = match constr {
                        Constraint::Filter { filter } => {
                            if filter.check_binding(&b, ocel) {
                                None
                            } else {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            }
                        }
                        Constraint::SizeFilter { filter } => {
                            if filter.check(&b, &child_res, ocel) {
                                None
                            } else {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            }
                        }
                        // For-all semantics!
                        Constraint::SAT { child_names } => {
                            let violated = child_names.iter().any(|child_name| {
                                if let Some(c_res) = child_res.get(child_name) {
                                    c_res.iter().any(|(_b, v)| v.is_some())
                                } else {
                                    true
                                }
                            });
                            if violated {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            } else {
                                None
                            }
                        }
                        // SAT with any (exists) semantics
                        Constraint::ANY { child_names } => {
                            let violated = child_names.iter().any(|child_name| {
                                if let Some(c_res) = child_res.get(child_name) {
                                    c_res.iter().all(|(_b, v)| v.is_some())
                                } else {
                                    true
                                }
                            });
                            if violated {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            } else {
                                None
                            }
                        }
                        Constraint::NOT { child_names } => {
                            let violated = child_names.iter().all(|child_name| {
                                if let Some(c_res) = child_res.get(child_name) {
                                    c_res.iter().any(|(_b, v)| v.is_none())
                                } else {
                                    true
                                }
                            });
                            if violated {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            } else {
                                None
                            }
                        }
                        Constraint::OR { child_names } => {
                            // println!("Child indices: {:?}, Children: {:?}", child_names, children);
                            let any_sat = child_names.iter().any(|child_name| {
                                if let Some(c_res) = child_res.get(child_name) {
                                    c_res.iter().all(|(_b, v)| v.is_none())
                                } else {
                                    true
                                }
                            });
                            if any_sat {
                                None
                            } else {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            }
                        }
                        Constraint::AND { child_names } => {
                            // println!("Child indices: {:?}, Children: {:?}", child_names, children);
                            let any_sat = child_names.iter().all(|child_name| {
                                if let Some(c_res) = child_res.get(child_name) {
                                    c_res.iter().all(|(_b, v)| v.is_none())
                                } else {
                                    true
                                }
                            });
                            if any_sat {
                                None
                            } else {
                                Some(ViolationReason::ConstraintNotSatisfied(constr_index))
                            }
                        }
                    };
                    if let Some(vr) = viol {
                        all_res.push((own_index, b.clone(), Some(vr)));
                        return BindingResult::Viol(b, vr, all_res);
                    }
                }
                all_res.push((own_index, b.clone(), None));
                BindingResult::Sat(b, all_res)
            })
            .collect();
        let recursive_calls_cancelled = x.is_cancelled();
        (re.into_par_iter()
            .fold(
                || (EvaluationResults::new(), Vec::new()),
                |(mut a, mut b), x| match x {
                    BindingResult::FilteredOutBySizeFilter(_binding, r) => {
                        a.extend(r);
                        (a, b)
                    }
                    BindingResult::Sat(binding, r) => {
                        a.extend(r);
                        b.push((binding, None));
                        (a, b)
                    }
                    BindingResult::Viol(binding, v, r) => {
                        a.extend(r);
                        b.push((binding, Some(v)));
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
            ),expanding_skipped_bindings || recursive_calls_cancelled)

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
    //     _ => todo!(),
    // }
    // }
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
        #[serde(default)]
        #[ts(optional)]
        #[serde(rename = "filterLabel")]
        filter_label: Option<FilterLabel>,
    },
    /// Object1 is associated with object2 (optionally through a qualifier)
    O2O {
        object: ObjectVariable,
        other_object: ObjectVariable,
        qualifier: Qualifier,
        #[serde(default)]
        #[ts(optional)]
        #[serde(rename = "filterLabel")]
        filter_label: Option<FilterLabel>,
    },
    /// Time duration betweeen event1 and event2 is in the specified interval (min,max) (given in Some(seconds); where None represents no restriction)
    TimeBetweenEvents {
        from_event: EventVariable,
        to_event: EventVariable,
        min_seconds: Option<f64>,
        max_seconds: Option<f64>,
    },
    NotEqual {
        var_1: Variable,
        var_2: Variable,
    },
    EventAttributeValueFilter {
        event: EventVariable,
        attribute_name: String,
        value_filter: ValueFilter,
    },
    ObjectAttributeValueFilter {
        object: ObjectVariable,
        attribute_name: String,
        at_time: ObjectValueFilterTimepoint,
        value_filter: ValueFilter,
    },
    BasicFilterCEL {
        cel: String,
    },
}

impl Filter {
    pub fn check_binding(&self, b: &Binding, ocel: &IndexLinkedOCEL) -> bool {
        match self {
            Filter::O2E {
                object,
                event,
                qualifier,
                filter_label: _,
            } => {
                let ob = b.get_ob(object, ocel).unwrap();
                let ev = b.get_ev(event, ocel).unwrap();

                ev.relationships.iter().any(|rel| {
                    rel.object_id == ob.id
                        && if let Some(q) = qualifier {
                            &rel.qualifier == q
                        } else {
                            true
                        }
                })
            }
            Filter::O2O {
                object,
                other_object,
                qualifier,
                filter_label: _,
            } => {
                let ob1 = b.get_ob(object, ocel).unwrap();
                let ob2 = b.get_ob(other_object, ocel).unwrap();
                ob1.relationships.iter().any(|rel| {
                    rel.object_id == ob2.id
                        && if let Some(q) = qualifier {
                            &rel.qualifier == q
                        } else {
                            true
                        }
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
            Filter::NotEqual { var_1, var_2 } => {
                let val_1 = b.get_any_index(var_1);
                let val_2 = b.get_any_index(var_2);
                !(val_1.is_none() || val_2.is_none() || val_1 == val_2)
            }
            Filter::EventAttributeValueFilter {
                event,
                attribute_name,
                value_filter,
            } => {
                let e_opt = b.get_ev(event, ocel);
                if let Some(e) = e_opt {
                    if let Some(attr) = e.attributes.iter().find(|at| &at.name == attribute_name) {
                        value_filter.check_value(&attr.value)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Filter::ObjectAttributeValueFilter {
                object,
                attribute_name,
                at_time,
                value_filter,
            } => {
                let o_opt = b.get_ob(object, ocel);
                if let Some(o) = o_opt {
                    match at_time {
                        ObjectValueFilterTimepoint::Always => o
                            .attributes
                            .iter()
                            .filter(|at| &at.name == attribute_name)
                            .all(|at| value_filter.check_value(&at.value)),
                        ObjectValueFilterTimepoint::Sometime => o
                            .attributes
                            .iter()
                            .filter(|at| &at.name == attribute_name)
                            .any(|at| value_filter.check_value(&at.value)),
                        ObjectValueFilterTimepoint::AtEvent { event } => {
                            if let Some(ev) = b.get_ev(event, ocel) {
                                // Find last attribute value update _before_ the event occured (or at the same time)
                                if let Some(last_val_before) = o
                                    .attributes
                                    .iter()
                                    .filter(|at| &at.name == attribute_name && at.time <= ev.time)
                                    .sorted_by_key(|x| x.time)
                                    .last()
                                {
                                    value_filter.check_value(&last_val_before.value)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                    }
                } else {
                    false
                }
            }
            Filter::BasicFilterCEL { cel } => {
                // let now = Instant::now();

                // println!("Took {:?}",now.elapsed());
                check_cel_predicate(cel, b, None, ocel)
            }
        }
    }
}

#[derive(TS, Debug, Clone, Serialize, Deserialize)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[serde(tag = "type")]

pub enum ValueFilter {
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Integer {
        // Prevent BigInt as TS/JS type
        // We do not really care about such large values + it messes with JSON
        #[ts(as = "Option<i32>")]
        min: Option<i64>,
        #[ts(as = "Option<i32>")]
        max: Option<i64>,
    },
    Boolean {
        is_true: bool,
    },
    String {
        is_in: Vec<String>,
    },
    Time {
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    },
}

impl ValueFilter {
    pub fn check_value(&self, val: &OCELAttributeValue) -> bool {
        match self {
            ValueFilter::Float { min, max } => match val {
                OCELAttributeValue::Float(v) => {
                    !min.is_some_and(|min_v| v < &min_v) && !max.is_some_and(|max_v| v > &max_v)
                }
                OCELAttributeValue::Integer(v) => {
                    !min.is_some_and(|min_v| (*v as f64) < min_v)
                        && !max.is_some_and(|max_v| (*v as f64) > max_v)
                }
                _ => false,
            },
            ValueFilter::Integer { min, max } => match val {
                OCELAttributeValue::Integer(v) => {
                    !min.is_some_and(|min_v| v < &min_v) && !max.is_some_and(|max_v| v > &max_v)
                }
                OCELAttributeValue::Float(v) => {
                    !min.is_some_and(|min_v| *v < (min_v as f64))
                        && !max.is_some_and(|max_v| *v > (max_v as f64))
                }
                _ => false,
            },
            ValueFilter::Boolean { is_true } => match val {
                OCELAttributeValue::Boolean(b) => is_true == b,
                _ => false,
            },
            ValueFilter::String { is_in } => match val {
                OCELAttributeValue::String(s) => is_in.contains(s),
                _ => false,
            },
            ValueFilter::Time { from, to } => match val {
                OCELAttributeValue::Time(v) => {
                    !from.is_some_and(|min_v| v < &min_v) && !to.is_some_and(|max_v| v > &max_v)
                }
                _ => false,
            },
        }
    }
}

#[derive(TS, Debug, Clone, Serialize, Deserialize)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[serde(tag = "type")]
pub enum ObjectValueFilterTimepoint {
    Always,
    Sometime,
    AtEvent { event: EventVariable },
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
    BindingSetEqual {
        child_names: Vec<NodeEdgeName>,
    },
    BindingSetProjectionEqual {
        child_name_with_var_name: Vec<(NodeEdgeName, Variable)>,
    },
    NumChildsProj {
        child_name: NodeEdgeName,
        var_name: Variable,
        min: Option<usize>,
        max: Option<usize>,
    },
    AdvancedCEL {
        cel: String,
    },
}

impl SizeFilter {
    pub fn check(
        &self,
        binding: &Binding,
        child_res: &HashMap<String, Vec<(Binding, Option<ViolationReason>)>>,
        ocel: &IndexLinkedOCEL,
    ) -> bool {
        match self {
            SizeFilter::NumChilds {
                child_name,
                min,
                max,
            } => {
                // println!("{child_index} {c} Min: {:?} Max: {:?} Len: {}",min,max,violations.len());
                if let Some(c_res) = child_res.get(child_name) {
                    if min.is_some_and(|min| c_res.len() < min) {
                        false
                    } else {
                        !max.is_some_and(|max| c_res.len() > max)
                    }
                } else {
                    false
                }
            }
            SizeFilter::NumChildsProj {
                child_name,
                var_name,
                min,
                max,
            } => {
                if let Some(c_res) = child_res.get(child_name) {
                    let set: HashSet<_> = c_res
                        .iter()
                        .flat_map(|(b, _)| b.get_any_index(var_name))
                        .collect();
                    if min.is_some_and(|min| set.len() < min) {
                        false
                    } else {
                        !max.is_some_and(|max| set.len() > max)
                    }
                } else {
                    false
                }
            }
            SizeFilter::BindingSetEqual { child_names } => {
                if child_names.is_empty() {
                    true
                } else if let Some(c_res) = child_res.get(&child_names[0]) {
                    let set: HashSet<_> = c_res.iter().map(|(binding, _)| binding).collect();
                    for other_c in child_names.iter().skip(1) {
                        if let Some(c2_res) = child_res.get(other_c) {
                            let set2: HashSet<_> =
                                c2_res.iter().map(|(binding, _)| binding).collect();
                            if set != set2 {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            SizeFilter::BindingSetProjectionEqual {
                child_name_with_var_name,
            } => {
                if child_name_with_var_name.is_empty() {
                    true
                } else if let Some(c_res) = child_res.get(&child_name_with_var_name[0].0) {
                    let set: HashSet<_> = c_res
                        .iter()
                        .map(|(binding, _)| match child_name_with_var_name[0].1 {
                            Variable::Event(e_var) => binding.get_ev_index(&e_var).map(|e| e.0),
                            Variable::Object(o_var) => binding.get_ob_index(&o_var).map(|o| o.0),
                        })
                        .collect();
                    for (other_c, var) in child_name_with_var_name.iter().skip(1) {
                        if let Some(c2_res) = child_res.get(other_c) {
                            let set2: HashSet<_> = c2_res
                                .iter()
                                .map(|(binding, _)| match var {
                                    Variable::Event(e_var) => {
                                        binding.get_ev_index(e_var).map(|e| e.0)
                                    }
                                    Variable::Object(o_var) => {
                                        binding.get_ob_index(o_var).map(|o| o.0)
                                    }
                                })
                                .collect();
                            if set != set2 {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            SizeFilter::AdvancedCEL { cel } => {
                check_cel_predicate(cel, binding, Some(child_res), ocel)
            }
        }
    }
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
    ANY { child_names: Vec<NodeEdgeName> },
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
                qualifier: _,
                filter_label: _,
            } => vec![Variable::Object(*object), Variable::Event(*event)]
                .into_iter()
                .collect(),
            Filter::O2O {
                object,
                other_object,
                qualifier: _,
                filter_label: _,
            } => vec![Variable::Object(*object), Variable::Object(*other_object)]
                .into_iter()
                .collect(),
            Filter::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds: _,
                max_seconds: _,
            } => vec![Variable::Event(*from_event), Variable::Event(*to_event)]
                .into_iter()
                .collect(),
            Filter::NotEqual { var_1, var_2 } => {
                vec![var_1.clone(), var_2.clone()].into_iter().collect()
            }
            Filter::EventAttributeValueFilter {
                event,
                attribute_name: _,
                value_filter: _,
            } => vec![Variable::Event(*event)].into_iter().collect(),
            Filter::ObjectAttributeValueFilter {
                object,
                attribute_name: _,
                at_time,
                value_filter: _,
            } => {
                let mut ret: HashSet<_> = vec![Variable::Object(*object)].into_iter().collect();
                if let ObjectValueFilterTimepoint::AtEvent { event } = at_time {
                    ret.insert(Variable::Event(*event));
                }
                ret
            }
            Filter::BasicFilterCEL { cel } => get_vars_in_cel_program(cel),
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
    // bool: reversed?
    BindObFromOb(ObjectVariable, ObjectVariable, Qualifier, bool),
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


#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InterMediateNode{
    pub event_vars: NewEventVariables,
    pub object_vars: NewObjectVariables,
   // pub relations: Vec<Relation> TODO: Need to define Relation enum
  //  pub inter_filter: Vec<InterFilter> TODO: Need to define InterFilter, also  decision which Filter to implement
  // Children Attributes will be represented in InterMediateTree

}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterMediateTree{
    pub nodes: Vec<InterMediateNode>,
    #[serde_as(as = "Vec<(_, _)>")]
    #[ts(as = "Vec<((usize, usize), String)>")]
    pub edge_names: HashMap<(usize, usize), String>,
}
