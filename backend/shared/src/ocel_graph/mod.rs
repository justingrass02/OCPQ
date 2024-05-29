use process_mining::ocel::ocel_struct::{OCELEvent, OCELObject};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::preprocessing::linked_ocel::{IndexLinkedOCEL, ObjectOrEventIndex};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GraphNode {
    Event(OCELEvent),
    Object(OCELObject),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GraphLink {
    source: String,
    target: String,
    qualifier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OCELGraph {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
pub struct OCELGraphOptions {
    max_distance: usize,
    root: String,
    root_is_object: bool,
    rels_size_ignore_threshold: usize,
    spanning_tree: bool,
}

pub fn get_ocel_graph<'a>(
    ocel: &'a IndexLinkedOCEL,
    options: OCELGraphOptions,
) -> Option<OCELGraph> {
    let root_index_opt = match options.root_is_object {
        true => {
            if let Some(o_index) = ocel.object_index_map.get(&options.root) {
                Some(ObjectOrEventIndex::Object(*o_index))
            } else {
                None
            }
        }

        false => {
            if let Some(e_index) = ocel.event_index_map.get(&options.root) {
                Some(ObjectOrEventIndex::Event(*e_index))
            } else {
                None
            }
        }
    };
    if let Some(root_index) = root_index_opt {
        let mut queue = vec![(root_index, 0)];
        let mut done_indices: Vec<ObjectOrEventIndex> = Vec::new();
        let mut expanded_arcs: Vec<(ObjectOrEventIndex, ObjectOrEventIndex)> = Vec::new();
        done_indices.push(root_index);
        let max_distance = options.max_distance;
        while !queue.is_empty() {
            let (index, distance) = queue.pop().unwrap();
            if distance < max_distance {
                if let Some(rels) = ocel.symmetric_rels.get(&index) {
                  // Check for rels_size_ignore_threshold but also continue if at the root node (root node always gets expanded)
                  if root_index == index || rels.len() < options.rels_size_ignore_threshold {
                      for r in rels {
                        if !done_indices.contains(r) {
                          expanded_arcs.push((index, *r));
                          queue.push((*r, distance + 1));
                          done_indices.push(*r);
                        } else if !options.spanning_tree {
                          expanded_arcs.push((index, *r));
                        }
                      }
                    }
                  }
            }
        }
        let nodes = done_indices
            .iter()
            .map(|i| match i {
                ObjectOrEventIndex::Object(o_index) => {
                    GraphNode::Object(ocel.ob_by_index(o_index).unwrap().clone())
                }
                ObjectOrEventIndex::Event(e_index) => {
                    GraphNode::Event(ocel.ev_by_index(e_index).unwrap().clone())
                }
            })
            .collect();
        let links = expanded_arcs
            .iter()
            .map(|(from, to)| {
                let from = ocel.ob_or_ev_by_index(*from).unwrap().cloned();
                let to = ocel.ob_or_ev_by_index(*to).unwrap().cloned();

                GraphLink {
                    source: from.get_id().clone(),
                    target: to.get_id().clone(),
                    qualifier: String::new(),
                }
            })
            .collect();
        Some(OCELGraph { nodes, links })
    } else {
        return None;
    }
}
