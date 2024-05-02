use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use process_mining::ocel::ocel_struct::{OCELEvent, OCELObject};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use ts_rs::TS;

use crate::preprocessing::linked_ocel::{IndexLinkedOCEL, EventIndex, ObjectIndex};
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
pub struct EventVariable(usize);
impl From<usize> for EventVariable {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectVariable(usize);
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

    pub fn get_ev_index(
        &self,
        ev_var: &EventVariable,
    ) -> Option<&EventIndex> {
        self.event_map.get(ev_var)
    }
    pub fn get_ob_index(
        &self,
        ob_var: &ObjectVariable
    ) -> Option<&ObjectIndex> {
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
    pub filter_constraint: Vec<FilterConstraint>,
}

#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingBoxTree {
    pub nodes: Vec<BindingBoxTreeNode>,
    #[serde_as(as = "Vec<(_, _)>")]
    #[ts(as = "Vec<((usize, usize), (Option<usize>, Option<usize>))>")]
    pub size_constraints: HashMap<(usize, usize), (Option<usize>, Option<usize>)>,
}

impl BindingBoxTree {
    pub fn evaluate(&self, ocel: &IndexLinkedOCEL) -> EvaluationResults {
        if let Some(root) = self.nodes.first() {
            let (mut ret, violation) = root.evaluate(0, 0, Binding::default(), self, ocel);
            ret.push((0, Binding::default(), violation));
            return ret;
        } else {
            return vec![];
        }
    }
}
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive( Debug, Clone, Serialize, Deserialize)]
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
    TooFewMatchingEvents,
    TooManyMatchingEvents,
    NoChildrenOfORSatisfied,
    LeftChildOfANDUnsatisfied,
    RightChildOfANDUnsatisfied,
    BothChildrenOfANDUnsatisfied,
    ChildrenOfNOTSatisfied,
    ChildNotSatisfied,
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
    ) -> (EvaluationResults, Option<ViolationReason>) {
        match self {
            BindingBoxTreeNode::Box(bbox, children) => {
                let expanded: Vec<Binding> = bbox.expand(vec![parent_binding.clone()], ocel);
                let (min_size, max_size) = tree
                    .size_constraints
                    .get(&(parent_index, own_index))
                    .cloned()
                    .unwrap_or_default();

                if min_size.is_some_and(|min| expanded.len() < min) {
                    return (vec![], Some(ViolationReason::TooFewMatchingEvents));
                    // return vec![(own_index, parent_binding,Some(ViolationReason::TooFewMatchingEvents))];
                }
                if max_size.is_some_and(|max| expanded.len() > max) {
                    return (vec![], Some(ViolationReason::TooManyMatchingEvents));
                    // return vec![(own_index, parent_binding,Some(ViolationReason::TooManyMatchingEvents))];
                }

                let (child_not_sat, ret) = expanded
                    .into_par_iter()
                    .flat_map_iter(|b| {
                        children.iter().map(move |c| {
                            let (mut c_res, violation) =
                                tree.nodes[*c].evaluate(*c, own_index, b.clone(), tree, ocel);
                            c_res.push((*c, b.clone(), violation));
                            if let Some(_x) = violation {
                                return (true, c_res);
                            } else {
                                return (false, c_res);
                            }
                        })
                    })
                    .reduce(
                        || (false, vec![]),
                        |(violated1, res1), (violated2, res2)| {
                            (
                                violated1 || violated2,
                                res1.iter().chain(res2.iter()).cloned().collect(),
                            )
                        },
                    );
                if child_not_sat {
                    return (ret, Some(ViolationReason::ChildNotSatisfied));
                } else {
                    return (ret, None);
                }
            }
            BindingBoxTreeNode::OR(i1, i2) => {
                let node1 = &tree.nodes[*i1];
                let node2 = &tree.nodes[*i2];

                let mut ret = vec![];

                let (res_1, violation_1) =
                    node1.evaluate(*i1, own_index, parent_binding.clone(), tree, ocel);

                ret.extend(res_1);
                ret.push((*i1, parent_binding.clone(), violation_1));

                let (res_2, violation_2) =
                    node2.evaluate(*i2, own_index, parent_binding.clone(), tree, ocel);

                ret.extend(res_2);
                ret.push((*i2, parent_binding.clone(), violation_2));

                if violation_1.is_some() && violation_2.is_some() {
                    return (ret, Some(ViolationReason::NoChildrenOfORSatisfied));
                }
                return (ret, None);
            }
            BindingBoxTreeNode::AND(i1, i2) => {
                let node1 = &tree.nodes[*i1];
                let node2 = &tree.nodes[*i2];

                let mut ret = vec![];

                let (res_1, violation_1) =
                    node1.evaluate(*i1, own_index, parent_binding.clone(), tree, ocel);

                ret.push((*i1, parent_binding.clone(), violation_1));
                ret.extend(res_1);
                let (res_2, violation_2) =
                    node2.evaluate(*i2, own_index, parent_binding.clone(), tree, ocel);
                ret.push((*i2, parent_binding.clone(), violation_2));
                ret.extend(res_2);

                if violation_1.is_some() {
                    return (ret, Some(ViolationReason::LeftChildOfANDUnsatisfied));
                } else if violation_2.is_some() {
                    return (ret, Some(ViolationReason::RightChildOfANDUnsatisfied));
                }
                return (ret, None);
            }
            BindingBoxTreeNode::NOT(i) => {
                let mut ret = vec![];
                let node = &tree.nodes[*i];

                let (res_c, violation_c) = node.evaluate(*i, own_index, parent_binding, tree, ocel);
                ret.extend(res_c);
                if violation_c.is_some() {
                    // NOT satisfied
                    return (ret, None);
                } else {
                    return (ret, Some(ViolationReason::ChildrenOfNOTSatisfied));
                }
            }
        }
    }
}


#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterConstraint {
    /// Object is associated with event (optionally through a qualifier)
    ObjectAssociatedWithEvent(ObjectVariable, EventVariable, Qualifier),
    /// Object1 is associated with object2 (optionally through a qualifier)
    ObjectAssociatedWithObject(ObjectVariable, ObjectVariable, Qualifier),
    /// Time duration betweeen event1 and event2 is in the specified interval (min,max) (given in Some(seconds); where None represents no restriction)
    TimeBetweenEvents(EventVariable, EventVariable, (Option<f64>, Option<f64>)),
}

impl FilterConstraint {
    pub fn get_involved_variables(&self) -> HashSet<Variable> {
        match self {
            FilterConstraint::ObjectAssociatedWithEvent(ov, ev, _) => {
                vec![Variable::Object(*ov), Variable::Event(*ev)]
                    .into_iter()
                    .collect()
            }
            FilterConstraint::ObjectAssociatedWithObject(ov1, ov2, _) => {
                vec![Variable::Object(*ov1), Variable::Object(*ov2)]
                    .into_iter()
                    .collect()
            }
            FilterConstraint::TimeBetweenEvents(ev1, ev2, _) => {
                vec![Variable::Event(*ev1), Variable::Event(*ev2)]
                    .into_iter()
                    .collect()
            }
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
    Filter(FilterConstraint),
}

//
// Display Implementations
//

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
