use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};
use crate::binding_box::{structs::{Constraint, Filter, SizeFilter}, BindingBoxTree};
use crate::binding_box::structs::NewEventVariables;
use crate::binding_box::structs::NewObjectVariables;
use crate::binding_box::structs::ObjectVariable;
use crate::binding_box::structs::EventVariable;
use crate::binding_box::structs::Qualifier;


#[derive(Clone)]
pub struct SQL_Parts{
    node: InterMediateNode,
    select_fields: Vec<String>,
    from_clauses: Vec<String>,
    where_clauses: Vec<String>,
    child_sql: Vec<(String,String)>,
    event_tables: HashMap<String,String>,
    object_tables: HashMap<String,String>,
    used_keys: HashSet<String>
}



// Implementation of the General translate to SQL function
pub fn translate_to_sql_shared(
    tree: BindingBoxTree
)-> String{

    // Erstelle Mapping der Tabellen Hashmaps event object String zu String
    
    let event_tables = map_to_event_tables(); // args can be done later, for now hardcoded for Management

    let object_tables = map_to_object_tables();


    
    //Step 1:  Extract Intermediate Representation
    let inter = convert_to_intermediate(tree);

    // Create SQL Struct

    let sql_parts = SQL_Parts{
        node: inter,
        select_fields: vec![],
        from_clauses: vec![],
        where_clauses: vec![],
        child_sql: vec![],
        event_tables: event_tables,
        object_tables: object_tables,
        used_keys: HashSet::new()
    };

    
    // Step 2: Translate the Intermediate Representation to SQL
    let result = translate_to_sql_from_intermediate(sql_parts);
    
    
    
    return result;
}


pub fn convert_to_intermediate(
    tree: BindingBoxTree
) -> InterMediateNode {

    // Recursive approach for each binding box, start with the root node
    let intermediate = bindingbox_to_intermediate(&tree, 0);

    
    return intermediate;
}


#[derive(Clone)]
pub struct InterMediateNode{
    pub event_vars: NewEventVariables,
    pub object_vars: NewObjectVariables,
    pub relations: Vec<Relation>, // O2O, E2O, TBE Basics have to be included
    pub constraints: Vec<Constraint>,
    pub children: Vec<(InterMediateNode, String)>,
    pub filter: Vec<Filter>,
    pub sizefilter: Vec<SizeFilter>
}


#[derive(Clone)]
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

    let (filter, sizefilter) = extract_filters(binding_box.filters.clone(), binding_box.size_filters.clone());

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
            sizefilter,
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

            Constraint::SizeFilter { filter } =>{
                match filter {
                    SizeFilter::NumChilds { child_name, min, max } =>{
                        result.push(constraint.clone());
                    }

                    _ => {
                // Ignore the other constraints
                }

                }
            }


            Constraint::Filter { filter } =>{
                result.push(constraint.clone());
            }

            Constraint::SAT { child_names }=>{
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
    filters: Vec<Filter>,
    size_filters: Vec<SizeFilter>
) -> (Vec<Filter>, Vec<SizeFilter>){
    let result = Vec::new();

    let mut resultSize = Vec::new();


    for size_filter in &size_filters{
        match size_filter{

            SizeFilter::NumChilds { child_name, min, max } =>{
                resultSize.push(size_filter.clone());
            }


            _ =>{
                resultSize.push(size_filter.clone()); // Take all for now
            }

        }
    }

    return (result, resultSize);
}



pub fn map_to_event_tables(
) -> HashMap<String, String>{

    let mut event_tables: HashMap<String, String> = HashMap::new();

    event_tables.insert("confirm order".to_string(), "ConfirmOrder".to_string());
    event_tables.insert("create package".to_string(), "CreatePackage".to_string());
    event_tables.insert("failed delivery".to_string(), "FailedDelivery".to_string());
    event_tables.insert("item out of stock".to_string(), "ItemOutOfStock".to_string());
    event_tables.insert("package delivered".to_string(), "PackageDelivered".to_string());
    event_tables.insert("pay order".to_string(), "PayOrder".to_string());
    event_tables.insert("payment reminder".to_string(), "PaymentReminder".to_string());
    event_tables.insert("pick item".to_string(), "PickItem".to_string());
    event_tables.insert("place order".to_string(), "PlaceOrder".to_string());
    event_tables.insert("reorder item".to_string(), "ReorderItem".to_string());
    event_tables.insert("send package".to_string(), "SendPackage".to_string());

    return event_tables;

}



pub fn map_to_object_tables(

) -> HashMap<String, String>{

    let mut object_tables: HashMap<String, String> = HashMap::new();

    object_tables.insert("customers".to_string(), "Customers".to_string());
    object_tables.insert("employees".to_string(), "Employees".to_string());
    object_tables.insert("items".to_string(), "Items".to_string());
    object_tables.insert("orders".to_string(), "Orders".to_string());
    object_tables.insert("packages".to_string(), "Packages".to_string());
    object_tables.insert("products".to_string(), "Products".to_string());


    return object_tables;

}




// End of Intermediate








// Start of SQL Translation




// Function which translates Intermediate to SQL
pub fn translate_to_sql_from_intermediate(
    mut sql_parts: SQL_Parts
) -> String {


    let mut used_keys = HashSet::new(); // Save all the Tables already created
    let mut  base_from = Vec::new();

    sql_parts.select_fields = construct_select_fields(&sql_parts);
    
    
    (base_from, used_keys) = construct_from_clauses(&sql_parts, used_keys);
    sql_parts.used_keys = used_keys;
    
    let (join_clauses, where_clauses) = construct_basic_operations(&sql_parts);
    sql_parts.where_clauses = where_clauses;

    
    let childs = construct_childstrings(&sql_parts);
    sql_parts.child_sql = childs;
    
    let child_strings: Vec<String> = sql_parts.child_sql
        .iter()
        .map(|(sql, _)| sql.clone())
        .collect();


    let filter_clauses = construct_filter_non_basic(&mut sql_parts);
    sql_parts.where_clauses.extend(filter_clauses);    

        let result = construct_result(
        &sql_parts,
        base_from,
        join_clauses,
        child_strings,
    );

    return result;
}



// Construct the resulting SQL query with tools given

pub fn construct_result(
    sql_parts: &SQL_Parts,
    base_from: Vec<String>,
    join_clauses: Vec<String>,
    child_strings: Vec<String>,

) -> String {
    let mut result = String::new();

    let mut from_section = String::new();

    


    // SELECT result
    result.push_str("SELECT ");



    result.push_str(&sql_parts.select_fields.join(","));   

    result.push_str("\n");

    // Handle Constraints and Childs here
    
    if  (!sql_parts.node.constraints.is_empty()){
        let child_constraint_string = construct_child_constraints(&sql_parts);
        result.push_str(&format!(",\n({}) AS satisfied \n", child_constraint_string));
    }




    // FROM result
    result.push_str(&format!("FROM {}\n", base_from.join(",\n")));
    
    for join in &join_clauses{
    result.push_str(&format!("{}\n",join));
    }



    // WHERE result

    if !sql_parts.where_clauses.is_empty() {
        result.push_str(&format!(
            "WHERE {}\n",
            sql_parts.where_clauses.join("\nAND ")
        ));
    }

    return result;
}



pub fn construct_select_fields(
     sql_parts: &SQL_Parts
) -> Vec<String> {
    let mut select_fields = Vec::new();


    for (obj_var, _) in &sql_parts.node.object_vars {
        select_fields.push(format!("O{}.ocel_id", obj_var.0));
    }

    for (event_var, _) in &sql_parts.node.event_vars {
        select_fields.push(format!("E{}.ocel_id", event_var.0));
    }

    return select_fields;
}


// Could make used_keys function argument and return type for children! Could also put E2O and O2O in there to make sure no double tables defined
pub fn construct_from_clauses(
    mut sql_parts: &SQL_Parts,
    mut used_keys: HashSet<String>
) -> (Vec<String>, HashSet<String>) {
    let mut from_clauses = Vec::new();


    for (obj_var, types) in &sql_parts.node.object_vars {
        for object_type in types {
            let key = format!("O{}", obj_var.0);
            if  used_keys.insert(key.clone()) {
                from_clauses.push(format!("object_{} AS {}", sql_parts.object_tables[object_type], key));
            }
        }
    }


    for (event_var, types) in &sql_parts.node.event_vars {
        for event_type in types {
            let key = format!("E{}", event_var.0);
            if  used_keys.insert(key.clone()) {
                from_clauses.push(format!("event_{} AS {}", sql_parts.event_tables[event_type] , key));
            }
        }
    }

    return (from_clauses,used_keys);
}




pub fn construct_basic_operations(
    sql_parts: &SQL_Parts
) -> (Vec<String>, Vec<String>) {
    let mut join_clauses = Vec::new();
    let mut where_clauses = Vec::new();

    for relation in &sql_parts.node.relations {
        match relation {
            Relation::E2O { event, object, .. } => {
                let eo_alias = format!("E2O{}{}", event.0, object.0);
                join_clauses.push(format!(
                    "INNER JOIN event_object AS {} ON {}.ocel_event_id = E{}.ocel_id AND {}.ocel_object_id = O{}.ocel_id",
                    eo_alias, eo_alias, event.0, eo_alias, object.0
                ));
            }

            Relation::O2O { object_1, object_2, .. } => {
                let o2o_alias = format!("O2O{}{}", object_1.0, object_2.0);
                join_clauses.push(format!(
                    "INNER JOIN object_object AS {} ON {}.ocel_source_id = O{}.ocel_id AND {}.ocel_target_id = O{}.ocel_id",
                    o2o_alias, o2o_alias, object_1.0, o2o_alias,object_2.0
                ));
            }

            Relation::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds,
                max_seconds,
            } => {
                where_clauses.push(format!("strftime('%s', E{}.ocel_time) <= strftime('%s', E{}.ocel_time)", from_event.0, to_event.0));
                if let Some(min) = min_seconds {
                    where_clauses.push(format!("strftime('%s', E{}.ocel_time) - strftime('%s', E{}.ocel_time) >= {}", to_event.0, from_event.0, min));
                }
                if let Some(max) = max_seconds {
                    where_clauses.push(format!("strftime('%s', E{}.ocel_time) - strftime('%s', E{}.ocel_time) <= {}", to_event.0, from_event.0, max));
                }
            }
        }
    }

    return (join_clauses, where_clauses);
}




pub fn construct_childstrings(sql_parts: &SQL_Parts) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for (inter_node, node_label) in &sql_parts.node.children {
        let mut child_sql_parts = SQL_Parts {
            node: inter_node.clone(),
            select_fields: vec![],
            from_clauses: vec![],
            where_clauses: vec![],
            child_sql: vec![],
            event_tables: sql_parts.event_tables.clone(),
            object_tables: sql_parts.object_tables.clone(),
            used_keys: sql_parts.used_keys.clone()
        };

        let child_sql = translate_to_sql_from_child(&mut child_sql_parts);
        result.push((child_sql, node_label.clone()));
    }

    return result;
}



//TODO in this function: Names of subq more different
pub fn construct_child_constraints(
    sql_parts: &SQL_Parts
) -> String { 
    let mut result_string = Vec::new();

    for (i, constraint) in sql_parts.node.constraints.iter().enumerate() {
        
        match constraint {
            
            
            Constraint::ANY { child_names } => {
                let mut parts = Vec::new();
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        parts.push(format!(
                            "EXISTS (SELECT 1 FROM ({}) AS subqC_{}_{}_{} WHERE subqC_{}_{}_{}.satisfied = 1 AND subqC_{}_{}_{}.count > 0 )",
                            child_sql, i, j,child_label, i, j,child_label, i, j,child_label.trim()
                        ));
                    }
                }
                if !parts.is_empty() {
                    result_string.push(format!("({})", parts.join(" OR ")));
                }
            }
            
            // Constraint AND ALL
            Constraint::AND { child_names } => {
                let mut parts = Vec::new();
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        parts.push(format!(
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{}_{}_{} WHERE subqC_{}_{}_{}.satisfied = 0)",
                            child_sql, i, j,child_label, i, j,child_label.trim()
                        ));
                    }
                }
                if !parts.is_empty() {
                    result_string.push(format!("({})", parts.join(" AND ")));
                }
            }
            Constraint::NOT { child_names } => {
                let mut parts = Vec::new();
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        parts.push(format!(
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{}_{}_{} WHERE subqC_{}_{}_{}.satisfied = 1)",
                            child_sql, i, j,child_label, i, j,child_label.trim()
                        ));
                    }
                }
                if !parts.is_empty() {
                    result_string.push(format!("({})", parts.join(" AND ")));
                }
            }
            
            Constraint::SAT { child_names } => {
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        result_string.push(format!(
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{}_{}_{} WHERE subqC_{}_{}_{}.satisfied = 0)",
                            child_sql, i, j,child_label, i, j,child_label.trim()
                        ));
                    }
                }
            }


            // Analog to AND ALl but now connect with OR 
            Constraint::OR { child_names } =>{
                let mut parts = Vec::new();
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        parts.push(format!(
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{}_{}_{} WHERE subqC_{}_{}_{}.satisfied = 0)",
                            child_sql, i, j,child_label, i, j,child_label.trim()
                        ));
                    }
                }
                if !parts.is_empty() {
                    result_string.push(format!("({})", parts.join(" OR ")));
                }
            }


            Constraint::SizeFilter { filter } => {
                if let SizeFilter::NumChilds { child_name, min, max } = filter {
                    for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                        if child_label == child_name {
                            let clause = match (min, max) {
                                (Some(min), Some(max)) =>
                                    format!("(SELECT subqC_{}_{}_{}.count FROM ({}) AS subqC_{}_{}_{}) BETWEEN {} AND {}", i, j,child_label.trim(), child_sql, i, j,child_label.trim(), min, max),
                                (Some(min), None) =>
                                    format!("(SELECT subqC_{}_{}_{}.count FROM ({}) AS subqC_{}_{}_{}) >= {}", i, j, child_label, child_sql, i, j, child_label.trim(), min),
                                (None, Some(max)) =>
                                    format!("(SELECT subqC_{}_{}_{}.count FROM ({}) AS subqC_{}_{}_{}) <= {}", i, j, child_label, child_sql, i, j,child_label.trim(), max),
                                (None, None) => continue,
                            };
                            result_string.push(clause);
                        }
                    }
                }
            }
            Constraint::Filter { filter } => {
                match filter {
                    Filter::O2E { object, event, .. } => {
                        result_string.push(format!(
                            "EXISTS (SELECT 1 FROM event_object AS E2O{}{} WHERE E2O{}{}.ocel_event_id = E{}.ocel_id AND E2O{}{}.ocel_object_id = O{}.ocel_id)",
                            event.0, object.0,
                            event.0, object.0,
                            event.0,
                            event.0, object.0, object.0
                        ));
                    }
                    Filter::O2O { object, other_object, .. } => {
                        result_string.push(format!(
                            "EXISTS (SELECT 1 FROM object_object AS O2O{}{} WHERE O2O{}{}.ocel_source_id = O{}.ocel_id AND O2O{}{}.ocel_target_id = O{}.ocel_id)",
                            object.0, other_object.0,
                            object.0, other_object.0,
                            object.0,
                            object.0, other_object.0, other_object.0
                        ));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    return  result_string.join(" AND ");
}




// Handling of Childs


pub fn translate_to_sql_from_child(
    sql_parts: &mut SQL_Parts
) -> String {
    let mut used_keys = HashSet::new();
    let mut base_from = Vec::new();

    (base_from, sql_parts.used_keys) = construct_from_clauses(&sql_parts, used_keys);
    let (join_clauses, where_clauses) = construct_basic_operations(&sql_parts);
    sql_parts.where_clauses = where_clauses;
    let childs = construct_childstrings(&sql_parts);
    sql_parts.child_sql = childs;

    let constraint_expr = construct_child_constraints( &sql_parts);

    let sub_condition = if constraint_expr.trim().is_empty() {
        "True".to_string()
    } else {
        constraint_expr
    };

    let select_fields = vec![
        "COUNT(*) AS count".to_string(),
        format!("CASE WHEN {} THEN 1 ELSE 0 END AS satisfied", sub_condition)
    ];



    // hier noch updaten
    sql_parts.select_fields = select_fields;

    return  construct_result_child(
        &sql_parts,
        base_from,
        join_clauses,
    );
}



pub fn construct_result_child(
    
    sql_parts: &SQL_Parts,
    base_from: Vec<String>,
    join_clauses: Vec<String>,

) -> String {
    let mut result = String::new();

    result.push_str("SELECT ");
    result.push_str(&sql_parts.select_fields.join(",\n"));
    result.push_str("\n");

    result.push_str(&format!("FROM {}\n", base_from.join(",\n")));
    for join in &join_clauses {
        result.push_str(&format!("{}\n", join));
    }

    if !sql_parts.where_clauses.is_empty() {
        result.push_str(&format!("WHERE {}\n", sql_parts.where_clauses.join("\nAND ")));
    }

    return result;
}



pub fn construct_filter_non_basic(
    sql_parts: &mut SQL_Parts
) -> Vec<String> {

    let mut result = Vec::new();

    for (i, sizefilter) in sql_parts.node.sizefilter.iter().enumerate(){

        match sizefilter{

            SizeFilter::NumChilds { child_name, min, max } =>{

                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                        if child_label == child_name {
                            let clause = match (min, max) {
                                (Some(min), Some(max)) =>
                                    format!("(SELECT subqF_{}_{}_{}.count FROM ({}) AS subqF_{}_{}_{}) BETWEEN {} AND {}", i, j,child_label.trim(), child_sql, i, j,child_label.trim(), min, max),
                                (Some(min), None) =>
                                    format!("(SELECT subqF_{}_{}_{}.count FROM ({}) AS subqF_{}_{}_{}) >= {}", i, j, child_label, child_sql, i, j, child_label.trim(), min),
                                (None, Some(max)) =>
                                    format!("(SELECT subqF_{}_{}_{}.count FROM ({}) AS subqF_{}_{}_{}) <= {}", i, j, child_label, child_sql, i, j,child_label.trim(), max),
                                (None, None) => continue,
                            };
                            result.push(clause);
                        }
                    }

            }


            _ =>{

            }

        }


    }    

   return result; 
}


// TODO
// 1. Do not use Table names more than once to avoid difficulties (check if e.g E1 can be used in two different Child Queries)
// 2. Event and Object Attributes
// 3. check where ocel_changed_field check is needed (may complicate with object attributes)
// 4. Do Union (optional)
