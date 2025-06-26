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
    let result = translate_to_sql_from_intermediate(&inter);
    
    
    
    return result;
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
    E2O{
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

    
    // Handle childs recursively with box to inter function
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
                result.push(Relation::E2O {
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


// Extract other meaningful filters (maybe these in relations are enough, but CEL could be considered)
pub fn extract_filters(
    filters: Vec<Filter>
) -> Vec<Filter>{
    let result = Vec::new();

    return result;
}


// Function which translates Intermediate to SQL
pub fn translate_to_sql_from_intermediate(
    node: &InterMediateNode
) -> String{
    
    // Create structure for every type of SQL we expect
    let mut select_fields: Vec<String> = Vec::new();
    let mut from_clauses: Vec<String> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();

    let mut var = 0; // using this variable to distinc event object tables, problem value gets lost when handling children

    // Create SELECT and FROM Clauses from Intermediate

        // First object Variables  TODO: index is shifted 1 in Binding Box could just +1 
    for (obj_var, types) in &node.object_vars{
        for object_type in types{
            select_fields.push(format!("O{}.ocel_id", obj_var.0));
            from_clauses.push(format!("object_{} AS O{}", object_type, obj_var.0));
        }
    }
    
        // Now Event Variables

    for (event_var, types) in &node.event_vars{
        for event_type in types{
            select_fields.push(format!("E{}.ocel_id", event_var.0));
            from_clauses.push(format!("event_{} AS E{}", event_type, event_var.0));
        }
    }   
    

    
    // Produce JOINS from E2O and O2O and put in from clauses

    for relation in &node.relations{

        match relation{
            Relation::E2O{event, object, qualifier, .. } => {
                from_clauses.push(format!("INNER JOIN event_object AS E2O{} ON E2O{}.ocel_event_id = E{}.ocel_id", var, var, event.0 )); // join event object table with Event
                from_clauses.push(format!("INNER JOIN E2O{}.ocel_object_id = O{}.ocel_id", var, object.0));
                var = var + 1;
            }
            
            Relation::O2O{object_1, object_2, qualifier, .. } => {
                from_clauses.push(format!("INNER JOIN object_object AS O2O{} ON O2O{}.ocel_source_id = O{}.ocel_id", var, var, object_1.0));
                from_clauses.push(format!("INNER JOIN O2O{}.target_id = O{}.ocel_id", var, object_2.0));
                var = var + 1;

            }
            Relation::TimeBetweenEvents { from_event , to_event, min_seconds, max_seconds } =>{
                match (min_seconds, max_seconds) {
                (None, None) => {
                    where_clauses.push(format!("E{}.ocel_time <= E{}.ocel_time", from_event.0, to_event.0));
                }
                (Some(min), None) => {
                    where_clauses.push(format!("E{}.ocel_time <= E{}.ocel_time", from_event.0, to_event.0));
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time >= {}", to_event.0, from_event.0, min));
                }
                (None, Some(max)) => {
                    where_clauses.push(format!("E{}.ocel_time <= E{}.ocel_time", from_event.0, to_event.0));
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time <= {:?}", to_event.0, from_event.0, max));
                }
                (Some(min), Some(max)) => {
                    where_clauses.push(format!("E{}.ocel_time <= E{}.ocel_time", from_event.0, to_event.0));
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time >= {}", to_event.0, from_event.0, min));
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time <= {}", to_event.0, from_event.0, max));
                }
    }
            }
            
           _ => {
                
            } 
        }
    }



    // Idea: Create a Vec that containts string output for the root childs
    let mut child_strings = Vec::new();

    // Creates SQL for children (should maybe use different function, since new Vars do not need be there, but experimental first here )
    for (inter_node, _node_label) in &node.children{
        let child_sql = translate_to_sql_from_intermediate(inter_node);
        child_strings.push(child_sql);
    }

    // Have to insert the Constraints and so on here i guess
    let mut result = format!(
        "SELECT {}({}) \nFROM {} \nWHERE {}",
        select_fields.join(", "),
        child_strings.join(", "),
        from_clauses.join(", "),
        where_clauses.join("AND ")
    );


    return result;
}

// TODO:
// Filter,Labels and Constraints, maybe start with functions, which make these tasks which will be implemented later
// How to handle multiple children and constraints to connect them
// If where is empty should not output it
// Child Select Klammern weg falls empty
// Could put more into helper functions to abstract more