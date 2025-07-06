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
    pub constraints: Vec<Constraint>,
    pub children: Vec<(InterMediateNode, String)>,
    pub filter: Vec<Filter>
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
    let relations = extract_basic_relations(binding_box.filters.clone());

    let constraints = extract_constraints(binding_box.constraints);

    
    // Handle childs recursively with box to inter function
    let mut children = Vec::new();

    let filter = extract_filters(binding_box.filters.clone());

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
            filter,
            constraints,
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
    let mut result = Vec::new();

    for constraint in &constraints{
        match constraint{
            Constraint::ANY { child_names } =>{
                result.push(constraint.clone());
            }

            Constraint::AND { child_names } =>{
                result.push(constraint.clone());
            }

            Constraint::NOT { child_names } => {
                result.push(constraint.clone());
            }
           
            _ => {
                // Ignore the other constraints
            }
        }
    }

    return result;
}


// Extract other meaningful filters (maybe these in relations are enough, but CEL could be considered)
pub fn extract_filters(
    filters: Vec<Filter>
) -> Vec<Filter>{
    let result = Vec::new();

    return result;
}


// End of Intermediate








// Start of SQL Translation




// Function which translates Intermediate to SQL
pub fn translate_to_sql_from_intermediate(
    node: &InterMediateNode
) -> String {

    let mut used_keys = HashSet::new(); // Save all the Tables already created
    let mut  base_from = Vec::new();

    let select_fields = construct_select_fields(node);
    (base_from, used_keys) = construct_from_clauses(node, used_keys);
    let (join_clauses, where_clauses) = construct_basic_operations(node);
    
    
    
    let childs = construct_childstrings(node);
    
    let child_strings: Vec<String> = childs
        .iter()
        .map(|(sql, _)| sql.clone())
        .collect();




    let result = construct_result(
        node,
        select_fields,
        base_from,
        join_clauses,
        where_clauses,
        child_strings,
        childs
    );

    return result;
}



// Construct the resulting SQL query with tools given

pub fn construct_result(
    node: &InterMediateNode,
    select_fields: Vec<String>,
    base_from: Vec<String>,
    join_clauses: Vec<String>,
    where_clauses: Vec<String>,
    child_strings: Vec<String>,
    childs: Vec<(String,String)>

) -> String {
    let mut result = String::new();

    let mut from_section = String::new();

    


    // SELECT result
    result.push_str(&format!(
        "SELECT {}\n",
        select_fields.join(", "),
    )); 


    // Handle Constraints and Childs here
    
    if !child_strings.is_empty(){
        let child_constraint_string = construct_child_constraints(&node,childs);
        result.push_str(&format!(",\n({})\n", child_constraint_string));
    }




    // FROM result
    result.push_str(&format!("FROM {}\n", base_from.join(",\n")));
    
    for join in &join_clauses{
    result.push_str(&format!("{}\n",join));
    }



    // WHERE result

    if !where_clauses.is_empty() {
        result.push_str(&format!(
            "WHERE {}\n",
            where_clauses.join("\nAND ")
        ));
    }

    return result;
}



pub fn construct_select_fields(
    node: &InterMediateNode
) -> Vec<String> {
    let mut select_fields = Vec::new();

    for (obj_var, _) in &node.object_vars {
        select_fields.push(format!("O{}.ocel_id", obj_var.0));
    }

    for (event_var, _) in &node.event_vars {
        select_fields.push(format!("E{}.ocel_id", event_var.0));
    }

    return select_fields;
}


// Could make used_keys function argument and return type for children! Could also put E2O and O2O in there to make sure no double tables defined
pub fn construct_from_clauses(
    node: &InterMediateNode,
    mut  used_keys:HashSet<String>
) -> (Vec<String>, HashSet<String>) {
    let mut from_clauses = Vec::new();


    for (obj_var, types) in &node.object_vars {
        for object_type in types {
            let key = format!("O{}", obj_var.0);
            if used_keys.insert(key.clone()) {
                from_clauses.push(format!("object_{} AS {}", object_type.replace(" ", "_"), key));
            }
        }
    }


    for (event_var, types) in &node.event_vars {
        for event_type in types {
            let key = format!("E{}", event_var.0);
            if used_keys.insert(key.clone()) {
                from_clauses.push(format!("event_{} AS {}", event_type.replace(" ", "_"), key));
            }
        }
    }

    return (from_clauses,used_keys);
}




pub fn construct_basic_operations(
    node: &InterMediateNode
) -> (Vec<String>, Vec<String>) {
    let mut join_clauses = Vec::new();
    let mut where_clauses = Vec::new();

    for relation in &node.relations {
        match relation {
            Relation::E2O { event, object, .. } => {
                let eo_alias = format!("E2O{}{}", event.0, object.0);
                join_clauses.push(format!(
                    "INNER JOIN event_object AS {} ON {}.ocel_event_id = E{}.ocel_id",
                    eo_alias, eo_alias, event.0
                ));
                where_clauses.push(format!(
                    "{}.ocel_object_id = O{}.ocel_id",
                    eo_alias, object.0
                ));
            }

            Relation::O2O { object_1, object_2, .. } => {
                let o2o_alias = format!("O2O{}{}", object_1.0, object_2.0);
                join_clauses.push(format!(
                    "INNER JOIN object_object AS {} ON {}.ocel_source_id = O{}.ocel_id",
                    o2o_alias, o2o_alias, object_1.0
                ));
                where_clauses.push(format!(
                    "{}.ocel_target_id = O{}.ocel_id",
                    o2o_alias, object_2.0
                ));
            }

            Relation::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds,
                max_seconds,
            } => {
                where_clauses.push(format!("E{}.ocel_time <= E{}.ocel_time", from_event.0, to_event.0));
                if let Some(min) = min_seconds {
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time >= {}", to_event.0, from_event.0, min));
                }
                if let Some(max) = max_seconds {
                    where_clauses.push(format!("E{}.ocel_time - E{}.ocel_time <= {}", to_event.0, from_event.0, max));
                }
            }
        }
    }

    return (join_clauses, where_clauses);
}




pub fn construct_childstrings(
    node: &InterMediateNode,
) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for (inter_node, node_label) in &node.children {
        let child_sql = translate_to_sql_from_child(inter_node);
        result.push((child_sql, node_label.clone()));
    }

    return result;
}



// Construct the Constraints connected with the Children SQL (idea put resulting string for constraints together at end maybe return Vec String)
pub fn construct_child_constraints(
    node: &InterMediateNode,
    childs: Vec<(String, String)>
) -> String {
    let mut result_string = Vec::new();

    // Iterate over all Constraints
    for constraint in &node.constraints {
        match constraint {
            
            
            
            // Constraint ANYSAT implementation
            Constraint::ANY { child_names: _ } => {
                let mut childs_in_constraints = Vec::new();

                for (child_sql, child_label) in &childs {
                    for (_child_node, edge_name) in &node.children {
                        if child_label == edge_name {
                            childs_in_constraints.push(format!("({}) >= 1 \n", child_sql));
                        }
                    }
                }

                // Combine with OR
                let mut or_constraint = childs_in_constraints.join(" OR \n");
                or_constraint = format!("({})", or_constraint);
                result_string.push(or_constraint);
            }



            // Constraint AND implementation (is wrong same problem as with ORALl)
            Constraint::AND { child_names: _ } => {
                let mut childs_in_constraints = Vec::new();

                for (child_sql, child_label) in &childs {
                    for (_child_node, edge_name) in &node.children {
                        if child_label == edge_name {
                            childs_in_constraints.push(format!("({}) >= 1 \n", child_sql));
                        }
                    }
                }

                // Combine with AND
                let mut and_constraint = childs_in_constraints.join(" AND \n");
                and_constraint = format!("({})", and_constraint);
                result_string.push(and_constraint);
            }



            // Constraint NOT
            Constraint::NOT { child_names: _ } =>{
                let mut childs_in_constraints = Vec::new();

                for (child_sql, child_label) in &childs {
                    for (_child_node, edge_name) in &node.children {
                        if child_label == edge_name {
                            childs_in_constraints.push(format!("({}) = 0 \n", child_sql));
                        }
                    }
                }

                // Combine with AND
                let mut or_constraint = childs_in_constraints.join(" AND \n");
                or_constraint = format!("({})", or_constraint);
                result_string.push(or_constraint);
            }







            _ => {
                // Ignore rest 
            }
        }
    }

    return result_string.join(" AND ");
}



pub fn translate_to_sql_from_child(
    node: &InterMediateNode
) -> String{
    let mut used_keys = HashSet::new(); // Save all the Tables already created
    let mut  base_from = Vec::new();

    let mut select_fields = Vec::new();
    select_fields.push("COUNT(*)".to_string());
    (base_from, used_keys) = construct_from_clauses(node, used_keys);
    let (join_clauses, where_clauses) = construct_basic_operations(node);
    
    
    
    let childs = construct_childstrings(node);
    
    let child_strings: Vec<String> = childs
        .iter()
        .map(|(sql, _)| sql.clone())
        .collect();




    let result = construct_result(
        node,
        select_fields,
        base_from,
        join_clauses,
        where_clauses,
        child_strings,
        childs
    );

    return result;
}


// TODO:
// How to exactly implement every Constraint
// Filter implementation by repeating Subquery in WHERE section?

