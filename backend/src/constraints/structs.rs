#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
enum DependencyType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "existsInTarget")]
    ExistsInTarget,
    #[serde(rename = "existsInSource")]
    ExistsInSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintType {
    #[serde(rename = "response")]
    Response,
    #[serde(rename = "unary-response")]
    UnaryResponse,
    #[serde(rename = "non-response")]
    NonResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeConstraint {
    #[serde(rename = "minSeconds")]
    pub min_seconds: f64,
    #[serde(rename = "maxSeconds")]
    pub max_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connection {
    #[serde(rename = "type")]
    pub connection_type: ConstraintType,
    #[serde(rename = "timeConstraint")]
    pub time_constraint: TimeConstraint,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectVariableO2O {
    #[serde(rename = "parentVariableName")]
    pub parent_variable_name: String,
    pub qualifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(rename = "initiallyBound")]
    pub initially_bound: bool,
    pub o2o: Option<ObjectVariableO2O>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectedVariable {
    pub variable: ObjectVariable,
    pub qualifier: Option<String>,
    pub bound: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CountConstraint {
    pub min: usize,
    pub max: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecondsRange {
    #[serde(rename = "minSeconds")]
    pub min_seconds: f64,
    #[serde(rename = "maxSeconds")]
    pub max_seconds: f64,
}
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventTreeNode {
    #[serde(rename = "eventType")]
    pub event_type: EventType,
    pub variables: Vec<SelectedVariable>,
    #[serde(rename = "countConstraint")]
    pub count_constraint: CountConstraint,
    #[serde(rename = "firstOrLastEventOfType")]
    pub first_or_last_event_of_type: Option<FirstOrLastEventOfType>,
    #[serde(rename = "waitingTimeConstraint")]
    pub waiting_time_constraint: Option<SecondsRange>,
    #[serde(rename = "numQualifiedObjectsConstraint")]
    pub num_qualified_objects_constraint: Option<HashMap<String, CountConstraint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreeNodeType {
    Event(EventTreeNode),
    /// OR: True if one of the two child nodes (given by String-ID) are true
    OR(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub parents: Vec<TreeNodeConnection>,
    pub children: Vec<TreeNodeConnection>,
    pub data: TreeNodeType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "exactly")]
    Exactly { value: String },
    #[serde(rename = "anyOf")]
    AnyOf { values: Vec<String> },
    #[serde(rename = "anyExcept")]
    AnyExcept { values: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FirstOrLastEventOfType {
    #[serde(rename = "first")]
    First,
    #[serde(rename = "last")]
    Last,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNodeConnection {
    pub id: String,
    /// Should be Some(...) connection for EventType connections
    pub connection: Option<Connection>,
    // #[serde(rename = "eventType")]
    // pub event_type: Option<EventType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoundValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdditionalBindingInfo {
    pub past_events: Vec<PastEventInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PastEventInfo {
    pub event_id: String,
    pub node_id: String,
}
