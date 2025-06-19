use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};
use crate::binding_box::{structs::{Constraint, Filter}, BindingBoxTree};
use crate::binding_box::structs::NewEventVariables;
use crate::binding_box::structs::NewObjectVariables;
use crate::binding_box::structs::ObjectVariable;
use crate::binding_box::structs::EventVariable;
use crate::binding_box::structs::Qualifier;



// Implementation of the General translate to SQL function
pub fn translate_to_sql_shared(
    tree: BindingBoxTree
)-> String{
    
    //Step 1:  Extract Intermediate Representation
    let inter = convert_to_intermediate(tree);

    
    // Step 2: Translate the Intermediate Representation to SQL
    let result = translate_to_sql_from_intermediate(inter);
    
    
    
    return "MoinMoin".to_string();
}


pub fn convert_to_intermediate(
    tree: BindingBoxTree
) -> InterMediateNode {

    // Recursive approach for each binding box, start with the root node
    let intermediate = bindingbox_to_intermediate(&tree, 0);

    
    return intermediate;
}



pub struct InterMediateNode{
    pub event_vars: NewEventVariables,
    pub object_vars: NewObjectVariables,
    pub relations: Vec<Relation>, // O2O, E2O, TBE Basics have to be included
    pub children: Vec<(InterMediateNode, String)>
  //  pub inter_filter: Vec<InterFilter>, TODO: Need to define InterFilter, also  decision which Filter to implement
  // pub inter_constraints: Vec<InterConstraint>
}

pub enum Relation{
    E20{
        event: EventVariable,
        object: ObjectVariable,
        qualifier: Qualifier
    },
    O2O{
        object_1: ObjectVariable,
        object_2: ObjectVariable,
        qualifier: Qualifier
    },
    TimeBetweenEvents {
        from_event: EventVariable,
        to_event: EventVariable,
        min_seconds: Option<f64>,
        max_seconds: Option<f64>,
    }
}

pub fn bindingbox_to_intermediate(
    tree: &BindingBoxTree,
    index: usize
) -> InterMediateNode{

    let node = &tree.nodes[index];

    let (binding_box, child_indices) = node.clone().to_box();

    // Copy event and object variables of the given binding box
    let event_vars = binding_box.new_event_vars.clone();
    let object_vars = binding_box.new_object_vars.clone();

    // Extract the relations we HAVE to translate to query language (O2O, E2O, TBE) could as mentioned split O2O, E2O and TBE
    let relations = extract_basic_relations(binding_box.filters);

    
    //Handle childs recursively with box to inter function
    let mut children = Vec::new();

    // Iterate over all BindingBoxes in tree
    for child_index in child_indices{
        let child_node = bindingbox_to_intermediate(tree, child_index);
        
        // Extract label names from edge_names
        let edge_name = tree
                .edge_names
                .get(&(index, child_index))
                .cloned()
                .unwrap_or_else(|| format!("unnamed_edge_{}_{}", index, child_index)); // Edge not there

            children.push((child_node, edge_name));
    }

    let result = InterMediateNode {
            event_vars,
            object_vars,
            relations,
            children,
        };

    return result;    

}

// Function to extract BASIC operations (E20,O2O,TBE)
pub fn extract_basic_relations(filters: Vec<Filter>) -> Vec<Relation> {
    let mut result = Vec::new();

    // Iterate over all filters and extract the ones we want to take into Intermediate Representation
    for filter in filters {
        //Here Filters we extract
        match filter {
            Filter::O2E { event, object, qualifier, .. } => {
                result.push(Relation::E20 {
                    event,
                    object,
                    qualifier,
                });
            }
            Filter::O2O { object, other_object, qualifier, .. } => {
                result.push(Relation::O2O {
                    object_1: object,
                    object_2: other_object,
                    qualifier,
                });
            }
            Filter::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds,
                max_seconds,
            } => {
                result.push(Relation::TimeBetweenEvents {
                    from_event,
                    to_event,
                    min_seconds,
                    max_seconds,
                });
            }
            _ => {
                // Ignore the other filters
            }
        }
    }

    return result;
}


// Extract Constraints we want to translate
pub fn extract_constraints(
    constraints :Vec<Constraint>
) -> Vec<Constraint>{
    let result = Vec::new();

    return result;
}



// TODO most likely recursive approach 
pub fn translate_to_sql_from_intermediate(
    inter: InterMediateNode
) -> String{
    return "translate_to_sql_from_intermediate".to_string();
}

// still todo IR : Event and Object Vars are numbers, covert them now to O1 for example or later
// Filter and Constraints, maybe start with functions, which make these tasks which will be implemented later
// After we could start with basics in SQL