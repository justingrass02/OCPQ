use std::{
    collections::{ HashMap, HashSet}
};
use crate::binding_box::{structs::{Constraint, Filter, ObjectValueFilterTimepoint, SizeFilter, ValueFilter}, BindingBoxTree};
use crate::binding_box::structs::NewEventVariables;
use crate::binding_box::structs::NewObjectVariables;
use crate::binding_box::structs::ObjectVariable;
use crate::binding_box::structs::EventVariable;
use crate::binding_box::structs::Qualifier;
use ts_rs::TS;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationToSQL{
    pub tree: BindingBoxTree,
    pub database_type: DatabaseType
}



#[derive(TS)]
#[ts(export, export_to = "../../../frontend/src/types/generated/")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DatabaseType {

SQLite,

DuckDB

}


#[derive(Clone)]
pub struct SqlParts{
    node: InterMediateNode,
    select_fields: Vec<String>,
    base_from: Vec<String>,
    join_clauses: Vec<String>,
    where_clauses: Vec<String>,
    child_sql: Vec<(String,String)>,
    event_tables: HashMap<String,String>,
    object_tables: HashMap<String,String>,
    used_keys: HashSet<String>,
    database_type: DatabaseType 
}



// Implementation of the General translate to SQL function
pub fn translate_to_sql_shared(
    tree: BindingBoxTree,
    database: DatabaseType
)-> String{

    
    //Step 1:  Extract Intermediate Representation
    let inter = convert_to_intermediate(tree);

   
   
    // Create SQL Struct

    let mut sql_parts = SqlParts{
        node: inter,
        select_fields: vec![],
        base_from: vec![],
        join_clauses: vec![],
        where_clauses: vec![],
        child_sql: vec![],
        event_tables: HashMap::new(),
        object_tables: HashMap::new(),
        used_keys: HashSet::new(),
        database_type: database ,
    };

    match sql_parts.database_type {

            DatabaseType::SQLite => {
                sql_parts.event_tables = map_to_event_tables_sqllite(); // args can be done later, for now hardcoded for Management

                sql_parts.object_tables = map_to_object_tables_sqllite();
            }

            DatabaseType::DuckDB => {
                sql_parts.event_tables = map_to_event_tables_duckdb();
                sql_parts.object_tables = map_to_object_tables_duckdb();
            }

        }
   
   
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
            Constraint::ANY { child_names: _} =>{
                result.push(constraint.clone());
            }

            Constraint::AND { child_names: _ } =>{
                result.push(constraint.clone());
            }

            Constraint::NOT { child_names: _ } => {
                result.push(constraint.clone());
            }


            Constraint::OR { child_names:_ } =>{
                result.push(constraint.clone());
            }


            Constraint::SizeFilter { filter } =>{
                match filter {
                    SizeFilter::NumChilds { child_name: _, min: _, max: _ } =>{
                        result.push(constraint.clone());
                    }

                    _ => {
                // Ignore the other constraints
                }

                }
            }


            Constraint::Filter { filter: _ } =>{
                result.push(constraint.clone());
            }

            Constraint::SAT { child_names: _ }=>{
                result.push(constraint.clone());
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
    let mut result = Vec::new();

    let mut result_size = Vec::new();


    for size_filter in &size_filters{
        match size_filter{

            SizeFilter::NumChilds { child_name:_, min:_, max:_ } =>{
                result_size.push(size_filter.clone());
            }


            _ =>{
                result_size.push(size_filter.clone()); // Take all for now
            }

        }
    }


    for filter in &filters{
        match filter{

            Filter::ObjectAttributeValueFilter { object:_, attribute_name:_, at_time:_, value_filter:_ } =>{
                result.push(filter.clone());
            }

            Filter::EventAttributeValueFilter { event:_, attribute_name:_, value_filter:_ } =>{
                result.push(filter.clone());
            }


            _=>{

            }


        }
    }



    return (result, result_size);
}



pub fn map_to_event_tables_sqllite(
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

    event_tables.insert("object".to_string(),"object".to_string() );



    event_tables.insert("A_Accepted".to_string(), "A_Accepted".to_string());
    event_tables.insert("A_Cancelled".to_string(), "A_Cancelled".to_string());
    event_tables.insert("A_Complete".to_string(), "A_Complete".to_string());
    event_tables.insert("A_Concept".to_string(), "A_Concept".to_string());
    event_tables.insert("A_Create Application".to_string(), "A_Create Application".to_string());
    event_tables.insert("A_Denied".to_string(), "A_Denied".to_string());
    event_tables.insert("A_Incomplete".to_string(), "A_Incomplete".to_string());
    event_tables.insert("A_Pending".to_string(), "A_Pending".to_string());
    event_tables.insert("A_Submitted".to_string(), "A_Submitted".to_string());
    event_tables.insert("A_Validating".to_string(), "A_Validating".to_string());

    event_tables.insert("O_Accepted".to_string(), "O_Accepted".to_string());
    event_tables.insert("O_Cancelled".to_string(), "O_Cancelled".to_string());
    event_tables.insert("O_Create Offer".to_string(), "O_Create Offer".to_string());
    event_tables.insert("O_Created".to_string(), "O_Created".to_string());
    event_tables.insert("O_Refused".to_string(), "O_Refused".to_string());
    event_tables.insert("O_Returned".to_string(), "O_Returned".to_string());
    event_tables.insert("O_Sent (mail and online)".to_string(), "O_Sent (mail and online)".to_string());
    event_tables.insert("O_Sent (online only)".to_string(), "O_Sent (online only)".to_string());

    event_tables.insert("W_Assess potential fraud".to_string(), "W_Assess potential fraud".to_string());
    event_tables.insert("W_Call after offers".to_string(), "W_Call after offers".to_string());
    event_tables.insert("W_Call incomplete files".to_string(), "W_Call incomplete files".to_string());
    event_tables.insert("W_Complete application".to_string(), "W_Complete application".to_string());
    event_tables.insert("W_Handle leads".to_string(), "W_Handle leads".to_string());
    event_tables.insert("W_Personal Loan collection".to_string(), "W_Personal Loan collection".to_string());
    event_tables.insert("W_Shorten completion".to_string(), "W_Shorten completion".to_string());
    event_tables.insert("W_Validate application".to_string(), "W_Validate application".to_string());



    return event_tables;

}



pub fn map_to_object_tables_sqllite(

) -> HashMap<String, String>{

    let mut object_tables: HashMap<String, String> = HashMap::new();

    object_tables.insert("customers".to_string(), "Customers".to_string());
    object_tables.insert("employees".to_string(), "Employees".to_string());
    object_tables.insert("items".to_string(), "Items".to_string());
    object_tables.insert("orders".to_string(), "Orders".to_string());
    object_tables.insert("packages".to_string(), "Packages".to_string());
    object_tables.insert("products".to_string(), "Products".to_string());

    object_tables.insert("object".to_string(),"object".to_string());





    object_tables.insert("Application".to_string(), "Application".to_string());
    object_tables.insert("Case_R".to_string(), "Case_R".to_string());
    object_tables.insert("Offer".to_string(), "Offer".to_string());
    object_tables.insert("Workflow".to_string(), "Workflow".to_string());


    return object_tables;

}



pub fn map_to_event_tables_duckdb(

) -> HashMap<String,String>{
    let mut event_tables: HashMap<String, String> = HashMap::new();

    event_tables.insert("confirm order".to_string(), "Confirm Order".to_string());
    event_tables.insert("create package".to_string(), "Create Package".to_string());
    event_tables.insert("failed delivery".to_string(), "Failed Delivery".to_string());
    event_tables.insert("item out of stock".to_string(), "Item Out Of Stock".to_string());
    event_tables.insert("package delivered".to_string(), "Package Delivered".to_string());
    event_tables.insert("pay order".to_string(), "Pay Order".to_string());
    event_tables.insert("payment reminder".to_string(), "Payment Reminder".to_string());
    event_tables.insert("pick item".to_string(), "Pick Item".to_string());
    event_tables.insert("place order".to_string(), "Place Order".to_string());
    event_tables.insert("reorder item".to_string(), "Reorder Item".to_string());
    event_tables.insert("send package".to_string(), "Send Package".to_string());

    event_tables.insert("object".to_string(),"object".to_string() );



    event_tables.insert("A_Accepted".to_string(), "A_Accepted".to_string());
    event_tables.insert("A_Cancelled".to_string(), "A_Cancelled".to_string());
    event_tables.insert("A_Complete".to_string(), "A_Complete".to_string());
    event_tables.insert("A_Concept".to_string(), "A_Concept".to_string());
    event_tables.insert("A_Create Application".to_string(), "A_Create Application".to_string());
    event_tables.insert("A_Denied".to_string(), "A_Denied".to_string());
    event_tables.insert("A_Incomplete".to_string(), "A_Incomplete".to_string());
    event_tables.insert("A_Pending".to_string(), "A_Pending".to_string());
    event_tables.insert("A_Submitted".to_string(), "A_Submitted".to_string());
    event_tables.insert("A_Validating".to_string(), "A_Validating".to_string());

    event_tables.insert("O_Accepted".to_string(), "O_Accepted".to_string());
    event_tables.insert("O_Cancelled".to_string(), "O_Cancelled".to_string());
    event_tables.insert("O_Create Offer".to_string(), "O_Create Offer".to_string());
    event_tables.insert("O_Created".to_string(), "O_Created".to_string());
    event_tables.insert("O_Refused".to_string(), "O_Refused".to_string());
    event_tables.insert("O_Returned".to_string(), "O_Returned".to_string());
    event_tables.insert("O_Sent (mail and online)".to_string(), "O_Sent (mail and online)".to_string());
    event_tables.insert("O_Sent (online only)".to_string(), "O_Sent (online only)".to_string());

    event_tables.insert("W_Assess potential fraud".to_string(), "W_Assess potential fraud".to_string());
    event_tables.insert("W_Call after offers".to_string(), "W_Call after offers".to_string());
    event_tables.insert("W_Call incomplete files".to_string(), "W_Call incomplete files".to_string());
    event_tables.insert("W_Complete application".to_string(), "W_Complete application".to_string());
    event_tables.insert("W_Handle leads".to_string(), "W_Handle leads".to_string());
    event_tables.insert("W_Personal Loan collection".to_string(), "W_Personal Loan collection".to_string());
    event_tables.insert("W_Shorten completion".to_string(), "W_Shorten completion".to_string());
    event_tables.insert("W_Validate application".to_string(), "W_Validate application".to_string());


    return event_tables;
}


pub fn map_to_object_tables_duckdb(

) -> HashMap<String,String>{
    let mut object_tables: HashMap<String, String> = HashMap::new();

    object_tables.insert("customers".to_string(), "Customers".to_string());
    object_tables.insert("employees".to_string(), "Employees".to_string());
    object_tables.insert("items".to_string(), "Items".to_string());
    object_tables.insert("orders".to_string(), "Orders".to_string());
    object_tables.insert("packages".to_string(), "Packages".to_string());
    object_tables.insert("products".to_string(), "Products".to_string());

    object_tables.insert("object".to_string(),"object".to_string());


    object_tables.insert("Application".to_string(), "Application".to_string());
    object_tables.insert("Case_R".to_string(), "Case_R".to_string());
    object_tables.insert("Offer".to_string(), "Offer".to_string());
    object_tables.insert("Workflow".to_string(), "Workflow".to_string());



    return object_tables;
}


// End of Intermediate








// Start of SQL Translation




// Function which translates Intermediate to SQL
pub fn translate_to_sql_from_intermediate(
     mut sql_parts: SqlParts
) -> String {


    sql_parts.select_fields = construct_select_fields(&sql_parts);
    
    
    sql_parts.base_from = construct_from_clauses(&mut sql_parts);
    
    (sql_parts.join_clauses, sql_parts.where_clauses) = construct_basic_operations(&mut sql_parts);

    let childs = construct_childstrings(&sql_parts);
    sql_parts.child_sql = childs;
    

    let filter_clauses = construct_filter_non_basic(&mut sql_parts);
    sql_parts.where_clauses.extend(filter_clauses);    

    for (obj_var, _types) in &sql_parts.node.object_vars {
            sql_parts.where_clauses.push(format!("O{}.ocel_changed_field IS NULL", obj_var.0));

    }


        let result = construct_result(
        &mut sql_parts,
    );

    return result;
}



// Construct the resulting SQL query with tools given

pub fn construct_result(
    sql_parts: &mut SqlParts,

) -> String {
    let mut result = String::new();


    
    // SELECT result
    result.push_str("SELECT ");



    result.push_str(&sql_parts.select_fields.join(","));   

    result.push_str("\n");

    // Handle Constraints and Childs here
    
    if  !sql_parts.node.constraints.is_empty() {
        let child_constraint_string = construct_child_constraints(sql_parts);
        result.push_str(&format!(
            ",\nCASE WHEN {} THEN 1 ELSE 0 END AS satisfied \n",
            child_constraint_string
        ));
    }



    let mut contains_relation = false;

    for filter in &sql_parts.node.relations{
        match filter{
            Relation::E2O { event:_, object:_, qualifier:_ } =>{
                contains_relation = true;
                break;

            }


            Relation::O2O { object_1:_, object_2:_, qualifier:_ } =>{
                contains_relation = true;
                break;
            }
        
            _ => {}
        
        }
    }



    // FROM result generate dummy if basefrom empty

    if  sql_parts.base_from.is_empty(){
        result.push_str("FROM (SELECT 1) as dummy");
    }else{

        if contains_relation{
            result.push_str(&format!("FROM {}\n", sql_parts.base_from.join("\n")));
        }else{
            result.push_str(&format!("FROM {}\n", sql_parts.base_from.join(",\n")))
        }


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
     sql_parts: &SqlParts
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

// 
pub fn get_object_type(
    node: InterMediateNode
    ,index: usize
) -> String{

    for (obj_var, types) in node.object_vars {
        if obj_var.0 == index{
            for object_type in types {
                return object_type.to_string();
        }

        }
    }
        return "no type found object".to_string()
}


pub fn get_event_type(
    node: InterMediateNode
    ,index: usize
) -> String{
    for (ev_var, types) in node.event_vars {
        if ev_var.0 == index{
            for event_type in types {
                return event_type.to_string();
            
        }

        }
    }

        return "no type found event".to_string()
}





// problem when creating table but Table was already created (missing entry)
pub fn construct_from_clauses(sql_parts: &mut SqlParts) -> Vec<String> {
    let mut from_clauses = Vec::new();
    let mut is_first_join = true;
    let mut counter = 0;


    for relation in &sql_parts.node.relations {
        match relation {
            Relation::E2O { event, object, qualifier: _ } => {
                let event_alias = format!("E{}", event.0);
                let object_alias = format!("O{}", object.0);
                let mut event_object_alias = format!("E2O{}", counter);

                while sql_parts.used_keys.contains(&event_object_alias) {
                    counter += 1;
                    event_object_alias = format!("E2O{}", counter);
                }
                sql_parts.used_keys.insert(event_object_alias.clone());

                if is_first_join { // first join to distinct if we have to use INNER JOIN first
                    if sql_parts.used_keys.contains(&event_alias) {
                        if sql_parts.used_keys.contains(&object_alias) {
                            from_clauses.push(format!("event_object AS {}", event_object_alias));
                            sql_parts.where_clauses.push(format!(
                                "{}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_alias
                            ));
                            sql_parts.where_clauses.push(format!(
                                "{}.ocel_object_id = {}.ocel_id",
                                event_object_alias, object_alias
                            ));
                        } else {
                            // event exists, object does not
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object.0)),
                                object_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_object_id = {}.ocel_id AND {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, object_alias, event_object_alias, event_alias
                            ));
                            sql_parts.used_keys.insert(object_alias.clone());
                        }
                    } else {
                        if sql_parts.used_keys.contains(&object_alias) {
                            // object table exists, event not
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_eventttables(sql_parts, &get_event_type(sql_parts.node.clone(), event.0)),
                                event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_object_id = {}.ocel_id AND {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, object_alias, event_object_alias, event_alias
                            ));
                            sql_parts.used_keys.insert(event_alias.clone());
                        } else {
                            // both not existing
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_eventttables(sql_parts, &get_event_type(sql_parts.node.clone(), event.0)),
                                event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_object_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object.0)),
                                object_alias,
                                event_object_alias,
                                object_alias
                            ));
                            sql_parts.used_keys.insert(object_alias.clone());
                            sql_parts.used_keys.insert(event_alias.clone());
                        }
                    }

                    is_first_join = false;
                } else {
                    if sql_parts.used_keys.contains(&event_alias) {
                        if sql_parts.used_keys.contains(&object_alias) {
                            // both table created
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_object_id = {}.ocel_id AND {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, object_alias, event_object_alias, event_alias
                            ));
                        } else {
                            // only event table
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_object_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object.0)),
                                object_alias,
                                event_object_alias,
                                object_alias
                            ));
                            sql_parts.used_keys.insert(object_alias.clone());
                        }
                    } else {
                        if sql_parts.used_keys.contains(&object_alias) {
                            // only object table created
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_object_id = {}.ocel_id",
                                event_object_alias, event_object_alias, object_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_event_id = {}.ocel_id",
                                map_eventttables(sql_parts, &get_event_type(sql_parts.node.clone(), event.0)),
                                event_alias,
                                event_object_alias,
                                event_alias
                            ));
                            sql_parts.used_keys.insert(event_alias.clone());
                        } else {
                            // both missing
                            from_clauses.push(format!(
                                "CROSS JOIN {} AS {}",
                                map_eventttables(sql_parts, &get_event_type(sql_parts.node.clone(), event.0)),
                                event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN event_object AS {} ON {}.ocel_event_id = {}.ocel_id",
                                event_object_alias, event_object_alias, event_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_object_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object.0)),
                                object_alias,
                                event_object_alias,
                                object_alias
                            ));
                            sql_parts.used_keys.insert(object_alias.clone());
                            sql_parts.used_keys.insert(event_alias.clone());
                        }
                    }
                }
            }

            Relation::O2O { object_1, object_2, qualifier: _ } => {
                let object1_alias = format!("O{}", object_1.0);
                let object2_alias = format!("O{}", object_2.0);
                let mut object_object_alias = format!("O2O{}", counter);

                while sql_parts.used_keys.contains(&object_object_alias) {
                    counter += 1;
                    object_object_alias = format!("O2O{}", counter);
                }
                sql_parts.used_keys.insert(object_object_alias.clone());

                if is_first_join {
                    if sql_parts.used_keys.contains(&object1_alias) {
                        if sql_parts.used_keys.contains(&object2_alias) {
                            from_clauses.push(format!("object_object AS {}", object_object_alias));
                            sql_parts.where_clauses.push(format!(
                                "{}.ocel_source_id = {}.ocel_id",
                                object_object_alias, object1_alias
                            ));
                            sql_parts.where_clauses.push(format!(
                                "{}.ocel_target_id = {}.ocel_id",
                                object_object_alias, object2_alias
                            ));
                        } else {
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_2.0)),
                                object2_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id AND {}.ocel_target_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias, object_object_alias, object2_alias
                            ));
                            sql_parts.used_keys.insert(object2_alias.clone());
                        }
                    } else {
                        if sql_parts.used_keys.contains(&object2_alias) {
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_1.0)),
                                object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id AND {}.ocel_target_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias, object_object_alias, object2_alias
                            ));
                            sql_parts.used_keys.insert(object1_alias.clone());
                        } else {
                            from_clauses.push(format!(
                                "{} AS {}",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_1.0)),
                                object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_target_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_2.0)),
                                object2_alias,
                                object_object_alias,
                                object2_alias
                            ));
                            sql_parts.used_keys.insert(object1_alias.clone());
                            sql_parts.used_keys.insert(object2_alias.clone());
                        }
                    }

                    is_first_join = false;
                } else {
                    if sql_parts.used_keys.contains(&object1_alias) {
                        if sql_parts.used_keys.contains(&object2_alias) {
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id AND {}.ocel_target_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias, object_object_alias, object2_alias
                            ));
                        } else {
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_target_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_2.0)),
                                object2_alias,
                                object_object_alias,
                                object2_alias
                            ));
                            sql_parts.used_keys.insert(object2_alias.clone());
                        }
                    } else {
                        if sql_parts.used_keys.contains(&object2_alias) {
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_target_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object2_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_source_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_1.0)),
                                object1_alias,
                                object_object_alias,
                                object1_alias
                            ));
                            sql_parts.used_keys.insert(object1_alias.clone());
                        } else {
                            from_clauses.push(format!(
                                "CROSS JOIN {} AS {}",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_1.0)),
                                object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN object_object AS {} ON {}.ocel_source_id = {}.ocel_id",
                                object_object_alias, object_object_alias, object1_alias
                            ));
                            from_clauses.push(format!(
                                "INNER JOIN {} AS {} ON {}.ocel_target_id = {}.ocel_id",
                                map_objecttables(sql_parts, &get_object_type(sql_parts.node.clone(), object_2.0)),
                                object2_alias,
                                object_object_alias,
                                object2_alias
                            ));
                            sql_parts.used_keys.insert(object2_alias.clone());
                            sql_parts.used_keys.insert(object1_alias.clone());
                        }
                    }
                }
            }

            _ => {}
        }
    }


    if is_first_join{// Does not contain E2O or O2O
        for (obj_var, types) in &sql_parts.node.object_vars {
        for object_type in types {
            let key = format!("O{}", obj_var.0);
            sql_parts.used_keys.insert(key.clone());
            from_clauses.push(format!("{} AS {}", map_objecttables(sql_parts, object_type), key));
            
        }
    }


    for (event_var, types) in &sql_parts.node.event_vars {
        for event_type in types {
            let key = format!("E{}", event_var.0);
            sql_parts.used_keys.insert(key.clone());
            from_clauses.push(format!("{} AS {}", map_eventttables(sql_parts, event_type) , key));
            
        }
    }
    }else{ // there might be relations, but there might be object tables which are not created

        for (obj_var, types) in &sql_parts.node.object_vars {
        for object_type in types {
            let key = format!("O{}", obj_var.0);
            sql_parts.used_keys.insert(key.clone());
            
             if !sql_parts.used_keys.contains(&key){
            from_clauses.push(format!(" CROSS JOIN {} AS {}", map_objecttables(sql_parts, object_type), key));
            sql_parts.used_keys.insert(key.clone());
             }
            
        }
    }


    for (event_var, types) in &sql_parts.node.event_vars {
        for event_type in types {
            let key = format!("E{}", event_var.0);
            if !sql_parts.used_keys.contains(&key){
            from_clauses.push(format!("CROSS JOIN{} AS {}", map_eventttables(sql_parts, event_type) , key));
            sql_parts.used_keys.insert(key.clone());
            }
            
        }
    }



    } 


    return from_clauses;
}






pub fn construct_basic_operations(
    sql_parts: &mut SqlParts
) -> (Vec<String>, Vec<String>) {
    let join_clauses = Vec::new();
    let mut where_clauses = sql_parts.where_clauses.clone();

    for relation in &sql_parts.node.relations {
        match relation {
            Relation::TimeBetweenEvents {
                from_event,
                to_event,
                min_seconds,
                max_seconds,
            } => {
                where_clauses.push(format!("{left_event} <= {right_event}",
                 left_event = map_timestamp_event(sql_parts, from_event.0),
                 right_event = map_timestamp_event(sql_parts, to_event.0)));
                if let Some(min) = min_seconds {
                    where_clauses.push(format!("{time_left} - {time_right} >= {min}", 
                    time_left = map_timestamp_event(sql_parts, to_event.0),
                    time_right = map_timestamp_event(sql_parts, from_event.0)));
                }
                if let Some(max) = max_seconds {
                    where_clauses.push(format!("{time_left} - {time_right} <= {max}",
                     time_left = map_timestamp_event(sql_parts, to_event.0),
                     time_right = map_timestamp_event(sql_parts, from_event.0)));
                }
           
           
           
            }
        
            _ => {}
            
        
        }

    }

    return (join_clauses, where_clauses);
}




pub fn construct_childstrings(sql_parts: &SqlParts) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for (inter_node, node_label) in &sql_parts.node.children {
        let mut child_sql_parts = SqlParts {
            node: inter_node.clone(),
            select_fields: vec![],
            base_from: vec![],
            join_clauses: vec![],
            where_clauses: vec![],
            child_sql: vec![],
            event_tables: sql_parts.event_tables.clone(),
            object_tables: sql_parts.object_tables.clone(),
            used_keys: sql_parts.used_keys.clone(),
            database_type: sql_parts.database_type,
        };




        let child_sql = translate_to_sql_from_child(&mut child_sql_parts);
        result.push((child_sql, node_label.clone()));
    }

    return result;
}



pub fn construct_child_constraints(
    sql_parts: &mut SqlParts
) -> String { 
    let mut result_string = Vec::new();

    for (i, constraint) in sql_parts.node.constraints.iter().enumerate() {
        
        match constraint {
            
            
            Constraint::ANY { child_names } => {
                let mut parts = Vec::new();
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_names.contains(child_label) {
                        parts.push(format!(
                            "(SELECT COUNT(*) FROM ({}) AS subqC_{i}_{j}_{label} WHERE subqC_{i}_{j}_{label}.satisfied = 1) >= 1",
                            child_sql,
                            i = i,
                            j = j,
                            label = child_label.trim()
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
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{iterator1}_{iterator2}_{label} WHERE subqC_{iterator1}_{iterator2}_{label}.satisfied = 0)",
                            child_sql,
                            iterator1 = i,
                            iterator2 = j,
                            label = child_label.trim()
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
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{iterator1}_{iterator2}_{label} WHERE subqC_{iterator1}_{iterator2}_{label}.satisfied = 1)",
                            child_sql,
                            iterator1 = i,
                            iterator2 = j,
                            label = child_label.trim()
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
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{iterator1}_{iterator2}_{label} WHERE subqC_{iterator1}_{iterator2}_{label}.satisfied = 0)",
                            child_sql,
                            iterator1 = i,
                            iterator2 = j,
                            label = child_label.trim()
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
                            "NOT EXISTS (SELECT 1 FROM ({}) AS subqC_{iterator1}_{iterator2}_{label} WHERE subqC_{iterator1}_{iterator2}_{label}.satisfied = 0)",
                            child_sql,
                            iterator1 = i,
                            iterator2 = j,
                            label = child_label.trim()
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
                        let count_expr = format!(
                            "(SELECT COUNT(*) FROM ({}) AS subqC_{i}_{j}_{label})",
                            child_sql,
                            i = i,
                            j = j,
                            label = child_label.trim()
                        );
                        let clause = match (min, max) {
                            (Some(min), Some(max)) => format!("{count_expr} BETWEEN {min} AND {max}"),
                            (Some(min), None) => format!("{count_expr} >= {min}"),
                            (None, Some(max)) => format!("{count_expr} <= {max}"),
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
                        let base_alias = format!("E2O{}{}", event.0, object.0);
                        let mut alias = base_alias.clone();
                        let mut counter = 1;

                        while sql_parts.used_keys.contains(&alias) {
                            alias = format!("{}_{}", base_alias, counter);
                            counter += 1;
                        }

                        sql_parts.used_keys.insert(alias.clone());
                        result_string.push(format!(
                            "EXISTS (SELECT 1 FROM {} AS {} WHERE {}.ocel_event_id = E{}.ocel_id AND {}.ocel_object_id = O{}.ocel_id)",
                            map_eventttables(sql_parts, "object")
                            ,alias, alias, event.0, alias, object.0
                        ));
                    }

                    Filter::O2O { object, other_object, .. } => {
                        let base_alias = format!("O2O{}{}", object.0, other_object.0);
                        let mut alias = base_alias.clone();
                        let mut counter = 1;

                        while sql_parts.used_keys.contains(&alias) {
                            alias = format!("{}_{}", base_alias, counter);
                            counter += 1;
                        }

                        sql_parts.used_keys.insert(alias.clone());

                        result_string.push(format!(
                            "EXISTS (SELECT 1 FROM {} AS {} WHERE {}.ocel_source_id = O{}.ocel_id AND {}.ocel_target_id = O{}.ocel_id)",
                            map_objecttables(sql_parts, "object")
                            ,alias, alias, object.0, alias, other_object.0
                        ));
                    }


                    Filter::TimeBetweenEvents { from_event, to_event, min_seconds, max_seconds } =>{

                        result_string.push(format!("{left_event} <= {right_event}",
                        left_event = map_timestamp_event(sql_parts, from_event.0),
                        right_event = map_timestamp_event(sql_parts, to_event.0)));
                        if let Some(min) = min_seconds {
                            result_string.push(format!("{time_left} - {time_right} >= {min}", 
                            time_left = map_timestamp_event(sql_parts, to_event.0),
                            time_right = map_timestamp_event(sql_parts, from_event.0)));
                        }
                        if let Some(max) = max_seconds {
                            result_string.push(format!("{time_left} - {time_right} <= {max}",
                            time_left = map_timestamp_event(sql_parts, to_event.0),
                            time_right = map_timestamp_event(sql_parts, from_event.0)));
                        }



                    }

                    _ => {}
                }
            }
        }
    }

    return  result_string.join(" AND ");
}




// Handling of Childs


pub fn translate_to_sql_from_child(
    sql_parts: &mut SqlParts
) -> String {

    sql_parts.base_from = construct_from_clauses(sql_parts);
    (sql_parts.join_clauses, sql_parts.where_clauses) = construct_basic_operations(sql_parts);
    
    let childs = construct_childstrings(&sql_parts);
    sql_parts.child_sql = childs;

    let constraint_expr = construct_child_constraints(sql_parts);

    let sub_condition = if constraint_expr.trim().is_empty() {
        "True".to_string()
    } else {
        constraint_expr
    };

    sql_parts.select_fields = vec![
            format!("CASE WHEN {} THEN 1 ELSE 0 END AS satisfied", sub_condition)
        ];



    return  construct_result_child(
        &sql_parts
    );
}



pub fn construct_result_child(
    
    sql_parts: &SqlParts,

) -> String {
    let mut result = String::new();

    result.push_str("SELECT ");
    result.push_str(&sql_parts.select_fields.join(",\n"));
    result.push_str("\n");

    if  sql_parts.base_from.is_empty(){
        result.push_str("FROM (SELECT 1) as dummy");
    }else{
    result.push_str(&format!("FROM {}\n", sql_parts.base_from.join("\n")));}
    
    
    
    if !sql_parts.where_clauses.is_empty() {
        result.push_str(&format!("WHERE {}\n", sql_parts.where_clauses.join("\nAND ")));
    }

    return result;
}



pub fn construct_filter_non_basic(
    sql_parts: &mut SqlParts
) -> Vec<String> {

    let mut result = Vec::new();

    for (i, sizefilter) in sql_parts.node.sizefilter.iter().enumerate(){

        match sizefilter{

            SizeFilter::NumChilds { child_name, min, max } => {
                for (j, (child_sql, child_label)) in sql_parts.child_sql.iter().enumerate() {
                    if child_label == child_name {
                        let clause = match (min, max) {
                            (Some(min), Some(max)) =>
                                format!(
                                    "(SELECT COUNT(*) FROM ({}) AS subqC_{i}_{j}_{label}) BETWEEN {} AND {}",
                                    child_sql, min, max,
                                    i = i,
                                    j = j,
                                    label = child_label.trim()
                                ),
                            (Some(min), None) =>
                                format!(
                                    "(SELECT COUNT(*) FROM ({}) AS subqC_{i}_{j}_{label}) >= {}",
                                    child_sql, min,
                                    i = i,
                                    j = j,
                                    label = child_label.trim()
                                ),
                            (None, Some(max)) =>
                                format!(
                                    "(SELECT COUNT(*) FROM ({}) AS subqC_{i}_{j}_{label}) <= {}",
                                    child_sql, max,
                                    i = i,
                                    j = j,
                                    label = child_label.trim()
                                ),
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

        for(i, filter) in sql_parts.node.filter.iter().enumerate(){

            match filter{


                Filter::EventAttributeValueFilter { event, attribute_name, value_filter } => {
                let col = format!("E{}.\"{}\"", event.0, attribute_name);

                let clause = match value_filter {
                    
                    
                    ValueFilter::String { is_in } => {
                        let values = is_in.iter()
                            .map(|v| format!("'{}'", v.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{} IN ({})", col, values)
                    },
                    
                    
                    ValueFilter::Boolean { is_true } => {
                        format!("{} = {}", col, is_true)
                    },
                    
                    ValueFilter::Integer { min, max } => {
                        let mut parts = vec![];
                        if let Some(min) = min {
                            parts.push(format!("{} >= {}", col, min));
                        }
                        if let Some(max) = max {
                            parts.push(format!("{} <= {}", col, max));
                        }
                        parts.join(" AND ")
                    },
                    
                    ValueFilter::Float { min, max } => {
                        let mut parts = vec![];
                        if let Some(min) = min {
                            parts.push(format!("{} >= {}", col, min));
                        }
                        if let Some(max) = max {
                            parts.push(format!("{} <= {}", col, max));
                        }
                        parts.join(" AND ")
                    },
                    
                    
                    
                    // prototyp cast probably necessary
                    ValueFilter::Time { from, to } => {
                        let mut parts = vec![];
                        if let Some(from) = from {
                            parts.push(format!("{time} >= '{from}'",
                            
                            time = map_timestamp(sql_parts, col.clone())));
                        }
                        if let Some(to) = to {
                            parts.push(format!("{time2} <= '{to}'",
                            
                            time2 = map_timestamp(sql_parts, col.clone())));
                        }
                        parts.join(" AND ")
                    },
                };

                result.push(clause);
            }



            Filter::ObjectAttributeValueFilter { object, attribute_name,at_time,value_filter } => {
                let object_alias = format!("O{}", object.0);
                let attr = attribute_name;
                let value_sql = match value_filter {
                    ValueFilter::String { is_in } => {
                        let values = is_in.iter()
                            .map(|v| format!("'{}'", v.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{}.{} IN ({})", object_alias, attr, values)
                    },
                    ValueFilter::Boolean { is_true } => {
                        format!("{}.{} = {}", object_alias,attr,is_true)
                    },
                    ValueFilter::Integer { min, max } => {
                        let mut parts = vec![];
                        if let Some(min) = min {
                            parts.push(format!("{}.{} >= {}", object_alias,attr, min));
                        }
                        if let Some(max) = max {
                            parts.push(format!("{}.{} <= {}", object_alias,attr,max));
                        }
                        parts.join(" AND ")
                    },
                    ValueFilter::Float { min, max } => {
                        let mut parts = vec![];
                        if let Some(min) = min {
                            parts.push(format!("{}.{} >= {}", object_alias,attr,min));
                        }
                        if let Some(max) = max {
                            parts.push(format!("{}.{} <= {}",object_alias,attr, max));
                        }
                        parts.join(" AND ")
                    },
                    
                    // need to see if this form conversion works as intended
                    ValueFilter::Time { from, to } => {
                        let mut parts = vec![];
                        if let Some(from) = from {
                            parts.push(format!("{time_left} >= {time_right}",
                            
                            time_left = map_timestamp(sql_parts, format!("{}.{}", object_alias, attr)),
                            time_right = map_timestamp(sql_parts, from.to_string())
                            ));
                        }
                        if let Some(to) = to {
                            parts.push(format!("{time_left} <= {time_right}",
                            time_left = map_timestamp(sql_parts, format!("{}.{}", object_alias, attr)),
                            time_right = map_timestamp(sql_parts, to.to_string())));
                        }
                        parts.join(" AND ")
                    },
                };


                let mut object_type = "";

                // bit scuffed but lets see (extracts object type)
                for (obj_var, types) in &sql_parts.node.object_vars {
                    for object_typer in types{
                        if obj_var.0 == object.0{
                             object_type = object_typer;
                        }
                    }
                    }



                // need for used_keys here?    
                let clause = match at_time {
                    ObjectValueFilterTimepoint::Sometime => {
                        let condition = value_sql;
                        format!(
                            "(EXISTS  SELECT 1\n FROM {otype} AS OA\n WHERE OA.ocel_id = {oid} AND {cond})",
                            otype = map_objecttables(sql_parts, object_type),
                            oid = format!("{}.ocel_id", object_alias),
                            cond = condition
                        )
                    },
                    
                    
                    ObjectValueFilterTimepoint::Always => {
                        let condition = value_sql;
                        format!(
                            "NOT EXISTS (SELECT 1\n FROM {otype} AS OA{iterator}\n WHERE OA{iterator}.ocel_id = {oid} AND NOT ({cond})
                            )",
                            iterator = i,
                            otype = map_objecttables(sql_parts, object_type),
                            oid = format!("{}.ocel_id", object_alias),
                            cond = condition
                        )
                    },
                    
                    
                    ObjectValueFilterTimepoint::AtEvent { event } => {
                        let event_time = format!("E{}.ocel_time", event.0);
                        let condition = value_sql;
                        format!(
                            "EXISTS (SELECT 1\n FROM {otype} AS OA{iterator}\n 
                            WHERE OA{iterator}.ocel_id = {oid} AND OA{iterator}.ocel_time = (SELECT MAX(OA2{iterator2}.ocel_time)\n 
                            FROM object_{otype} AS OA2{iterator2}\n WHERE OA2{iterator2}.ocel_id = {oid} AND {time_left}  <= {time_right}   ) AND {cond})",
                            iterator = i,
                            iterator2 = i*3,
                            time_left = map_timestamp(sql_parts, format!("OA2{}.ocel_time", i*3)),
                            time_right = map_timestamp(sql_parts, event_time ),
                            otype = map_objecttables(sql_parts, object_type),
                            oid = format!("{}.ocel_id", object_alias),
                            cond = condition
                        )
                    }


                };

                result.push(clause);
            }

            _ => {
                
            }



            }


        }
        
   return result; 
}



pub fn map_objecttables(
    sql_parts: &SqlParts,
    object_type: &str

) -> String {

    match sql_parts.database_type{

        // Case SQLLite
        DatabaseType::SQLite =>{
             return format!("object_{}", sql_parts.object_tables[object_type]);
        }


        //Case DuckDB
        DatabaseType::DuckDB =>{
            return format!("\"object_{}\"", sql_parts.object_tables[object_type]);
        }


    }



}

pub fn map_eventttables(
    sql_parts: &SqlParts,
    event_type: &str

) -> String {

    match sql_parts.database_type{

        // Case SQLLite
        DatabaseType::SQLite =>{
            return format!("event_{}", sql_parts.event_tables[event_type]);
        }


        //Case DuckDB
        DatabaseType::DuckDB =>{
            return format!("\"event_{}\"", sql_parts.event_tables[event_type]);
        }


    }



}


pub fn map_timestamp_event(
    sql_parts: &SqlParts,
    event_count: usize
) -> String{

    match sql_parts.database_type{

        DatabaseType::SQLite =>{
            return format!("strftime('%s', E{}.ocel_time)", event_count);
        }



        DatabaseType::DuckDB => {
            return format!("EPOCH(E{}.ocel_time)", event_count);
        }

    }


}

pub fn map_timestamp(
    sql_parts: &SqlParts,
    alias: String
) -> String {

    match sql_parts.database_type{

        DatabaseType::SQLite =>{
            return format!("strftime('%s', {})", alias)
        }


        DatabaseType::DuckDB =>{
             return format!("EPOCH({})", alias);
        }

    }

}


#[test]

fn export_sql_queries(){

use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

let query_names = ["Q1","Q2","Q3","Q4","Q5","Q6",
"Q7"];

let base_path = PathBuf::from_str("C:\\Users\\justi\\Desktop\\ocpq-eval").unwrap();


for query in query_names {

let tree_path = base_path.join(query).join("ocpq-tree.json");

for db_type in [DatabaseType::SQLite, DatabaseType::DuckDB]
{ 

let tree = serde_json::from_reader(File::open(&tree_path).unwrap()).unwrap();

let sql = translate_to_sql_shared(tree, db_type);

let sql_export_path = base_path.join(query).join(format!("auto-sql-{db_type:?}.txt"));

let mut sql_export_file = File::create(sql_export_path).unwrap();

write!(sql_export_file,"{sql}").unwrap();

}

}

}






// Cypher Translation 


pub struct CypherParts{
    node:  InterMediateNode,
    match_clauses: Vec<String>,
    return_clauses: Vec<String>,
    used_alias: HashSet<String>,
    event_tables: HashMap<String,String>,
    object_tables: HashMap<String,String>,
}




pub fn translate_to_cypher_shared(
   tree: BindingBoxTree ) -> String{
    
    // Convert to Intermediate

    let inter = convert_to_intermediate(tree);



    let mut cypher_parts = CypherParts{
    node: inter,    
    match_clauses: vec![],
    return_clauses: vec![],
    used_alias: HashSet::new(),
    event_tables: HashMap::new(),
    object_tables: HashMap::new()
    };


    get_event_table_cypher(&mut cypher_parts);

    get_object_table_cypher(&mut cypher_parts);

    let result = convert_to_cypher_from_inter(&mut cypher_parts);


    return result;



   }

// For root node in particular, could make other functions for child queries later
pub fn convert_to_cypher_from_inter(
   cypher_parts: &mut CypherParts
)  -> String{
    
    construct_match_clauses(cypher_parts);
    
    

    construct_return_clauses(cypher_parts);
    
    
    


    let result = construct_result_cypher(cypher_parts);

    return result;
    
    }  



// Start with E2O and O2O    
pub fn construct_match_clauses(
     cypher_parts: &mut CypherParts
){


    for relation in &cypher_parts.node.relations{
        match relation{

            Relation::E2O { event, object, qualifier:_ } =>{

                let event_alias = format!("e{}", event.0);
                let object_alias = format!("o{}", object.0);

                let event_object_alias = "E2O".to_string();

                let event_type = get_event_type(cypher_parts.node.clone(), event.0);
                let object_type = get_object_type(cypher_parts.node.clone(), object.0);


                let mapped_event_type = &cypher_parts.event_tables[&event_type.clone()];
                let mapped_object_type = &cypher_parts.object_tables[&object_type.clone()];


                cypher_parts.used_alias.insert(event_alias.clone());
                cypher_parts.used_alias.insert(object_alias.clone());

                cypher_parts.match_clauses.push(format!("({event_alias}:{mapped_event_type})-[:{event_object_alias}]->({object_alias}:{mapped_object_type})", 
                
            ));



            }


            Relation::O2O { object_1, object_2, qualifier:_ } => {

                let object1_alias = format!("o{}", object_1.0);
                let object2_alias = format!("o{}", object_2.0);

                let object_object_alias = "O2O".to_string();

                

                let object1_type = get_object_type(cypher_parts.node.clone(), object_1.0);
                let object2_type = get_object_type(cypher_parts.node.clone(), object_2.0);

                let mapped_object1_type =  &cypher_parts.object_tables[&object1_type.clone()];
                let mapped_object2_type =  &cypher_parts.object_tables[&object2_type.clone()];

                cypher_parts.used_alias.insert(object1_alias.clone());
                cypher_parts.used_alias.insert(object2_alias.clone());

                cypher_parts.match_clauses.push(format!("({object1_alias}:{mapped_object1_type})-[:{object_object_alias}]->({object2_alias}:{mapped_object2_type})"));


            }



            _ =>{

            }
        }
    }



}


pub fn get_event_table_cypher(
    cypher_parts: &mut CypherParts
){
    cypher_parts.event_tables.insert("confirm order".to_string(), "confirmorder".to_string());
    cypher_parts.event_tables.insert("create package".to_string(), "createpackage".to_string());
    cypher_parts.event_tables.insert("failed delivery".to_string(), "faileddelivery".to_string());
    cypher_parts.event_tables.insert("item out of stock".to_string(), "itemoutofstock".to_string());
    cypher_parts.event_tables.insert("package delivered".to_string(), "packagedelivered".to_string());
    cypher_parts.event_tables.insert("pay order".to_string(), "payorder".to_string());
    cypher_parts.event_tables.insert("payment reminder".to_string(), "paymentreminder".to_string());
    cypher_parts.event_tables.insert("pick item".to_string(), "pickitem".to_string());
    cypher_parts.event_tables.insert("place order".to_string(), "placeorder".to_string());
    cypher_parts.event_tables.insert("reorder item".to_string(), "reorderitem".to_string());
    cypher_parts.event_tables.insert("send package".to_string(), "sendpackage".to_string());
}



pub fn get_object_table_cypher(
    cypher_parts: &mut CypherParts
){
    cypher_parts.object_tables.insert("customers".to_string(), "customers".to_string());
    cypher_parts.object_tables.insert("employees".to_string(), "employees".to_string());
    cypher_parts.object_tables.insert("items".to_string(), "items".to_string());
    cypher_parts.object_tables.insert("orders".to_string(), "orders".to_string());
    cypher_parts.object_tables.insert("packages".to_string(), "packages".to_string());
    cypher_parts.object_tables.insert("products".to_string(), "products".to_string());
}


// Construct return clauses, at the moment only event and object ids
pub fn construct_return_clauses(
    cypher_parts: &mut CypherParts
){

    for (obj_var, _) in &cypher_parts.node.object_vars {
        cypher_parts.return_clauses.push(format!("o{}.id", obj_var.0));
    }

    for (event_var, _) in &cypher_parts.node.event_vars {
        cypher_parts.return_clauses.push(format!("e{}.id", event_var.0));
    }


}



pub fn construct_result_cypher(
    cypher_parts: &mut CypherParts
) -> String{

    let mut result = String::new();
    
    // MATCH
    for match_clause in &cypher_parts.match_clauses{
        result.push_str(&format!("MATCH {match_clause}\n"));
    }



    // Return

    result.push_str(&format!("RETURN {}", cypher_parts.return_clauses.join(",")));

    return result;

}
