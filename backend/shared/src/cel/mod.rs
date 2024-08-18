use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use cel_interpreter::{extractors::This, Context, FunctionContext, Program, ResolveResult, Value};
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use once_cell::sync::Lazy;
use process_mining::ocel::ocel_struct::OCELAttributeValue;

use crate::{
    binding_box::{
        structs::{EventVariable, ObjectVariable, Variable},
        Binding,
    },
    preprocessing::linked_ocel::{IndexLinkedOCEL, OCELNodeRef},
};

#[cfg(test)]
mod tests {
    

    

    use process_mining::import_ocel_json_from_path;

    use crate::{
        binding_box::structs::{Binding, EventVariable, ObjectVariable},
        preprocessing::linked_ocel::{
            link_ocel_info, EventIndex,
            ObjectIndex,
        },
    };

    use super::evaluate_cel;

    #[test]
    fn test_cel() {
        // let p = Program::compile("o1.type() in ['orders','items'] && get_attr(o1,'price') <= 100")
        //     .unwrap();
        // let mut context = Context::default();
        let ocel = import_ocel_json_from_path(
            "/home/aarkue/doc/projects/ocedeclare/backend/data/order-management.json",
        )
        .unwrap();
        let ocel = link_ocel_info(ocel);
        let binding = Binding::default()
            .expand_with_ev(EventVariable(0), EventIndex(0))
            .expand_with_ob(ObjectVariable(0), ObjectIndex(2));

        let sat = evaluate_cel(
            "o1.type() in ['orders','items'] && get_attr(o1,'price') <= 100",
            &binding,
            &ocel,
        );
        println!("SAT: {sat}");
    }
}

fn string_to_var(s: &str) -> Variable {
    let (typ, num) = s.split_at(1);
    let num = num.parse::<usize>().map(|v| v - 1).unwrap_or_default();
    if typ == "o" {
        Variable::Object(ObjectVariable(num))
    } else {
        Variable::Event(EventVariable(num))
    }
}

struct RawBindingContextPtr<'a, T>(*mut &'a T);

unsafe impl<'a, T> Send for RawBindingContextPtr<'a, T> {}
unsafe impl<'a, T> Sync for RawBindingContextPtr<'a, T> {}
impl<'a, T> Clone for RawBindingContextPtr<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T> Copy for RawBindingContextPtr<'a, T> {}

fn var_string_to_val<'a, 'b, 's>(
    s: &'s str,
    binding: &'b Binding,
    ocel: &'a IndexLinkedOCEL,
) -> Option<OCELNodeRef<'a>> {
    let var = string_to_var(s);
    binding
        .get_any_index(&var)
        .and_then(|i| ocel.ob_or_ev_by_index(i))
}

unsafe fn var_string_to_val_raw<'a, 'b, 's>(
    s: &'s str,
    binding: RawBindingContextPtr<Binding>,
    ocel: RawBindingContextPtr<'a, IndexLinkedOCEL>,
) -> Option<OCELNodeRef<'a>> {
    var_string_to_val(s, binding.0.as_ref().unwrap(), ocel.0.as_ref().unwrap())
}

pub static CEL_PROGRAM_CACHE: Lazy<RwLock<HashMap<String, Program>>> = Lazy::new(|| {
    let m = HashMap::new();
    RwLock::new(m)
});

pub fn lazy_compile_and_insert_into_cache(cel: &str) {
    let already_in_cache = CEL_PROGRAM_CACHE.read().unwrap().contains_key(cel);
    if !already_in_cache {
        let program = Program::compile(cel).unwrap();
        let mut w_lock = CEL_PROGRAM_CACHE.write().unwrap();
        w_lock.insert(cel.to_string(), program);
    }
}

pub fn evaluate_cel<'a>(cel: &str, binding: &'a Binding, ocel: &'a IndexLinkedOCEL) -> bool {
    // let now = Instant::now();
    lazy_compile_and_insert_into_cache(cel);
    let cache_read = CEL_PROGRAM_CACHE.read().unwrap();
    let p = cache_read.get(cel).unwrap();

    // println!("Program compiled: {:?}", now.elapsed());

    let mut context: Context<'a> = Context::default();
    for e_var in binding.event_map.keys() {
        let name = format!("e{}", e_var.0 + 1);
        context.add_variable(name.clone(), name).unwrap();
    }
    for o_var in binding.object_map.keys() {
        let name = format!("o{}", o_var.0 + 1);
        context.add_variable(name.clone(), name).unwrap();
    }

    // println!("Context added: {:?}", now.elapsed());

    let ocel_raw = RawBindingContextPtr(unsafe {
        std::mem::transmute::<*mut &'a IndexLinkedOCEL, *mut &'static IndexLinkedOCEL>(
            Box::into_raw(Box::new(ocel)),
        )
    });
    let binding_raw = RawBindingContextPtr(unsafe {
        std::mem::transmute::<*mut &'a Binding, *mut &'static Binding>(Box::into_raw(Box::new(
            binding,
        )))
    });

    context.add_function(
        "type",
        move |ftx: &FunctionContext, This(variable): This<Arc<String>>| -> ResolveResult {
            let val = unsafe { var_string_to_val_raw(&variable, binding_raw, ocel_raw) };
            let res = match val {
                Some(val_ref) => {
                    let ocel_type = match val_ref {
                        OCELNodeRef::Event(ev) => &ev.event_type,
                        OCELNodeRef::Object(ob) => &ob.object_type,
                    };
                    Ok(ocel_type.clone().into())
                }

                None => ftx.error("Event or Object not found.").into(),
            };
            res
        },
    );

    context.add_function(
        "attr",
        move |ftx: &FunctionContext,
              This(variable): This<Arc<String>>,
              attr_name: Arc<String>|
              -> ResolveResult {
            let val = unsafe { var_string_to_val_raw(&variable, binding_raw, ocel_raw) };
            let res = match val {
                Some(val_ref) => {
                    let attr_val = match val_ref {
                        OCELNodeRef::Event(ev) => ev
                            .attributes
                            .iter()
                            .filter(|a| &a.name == attr_name.as_ref())
                            .next()
                            .map(|a| &a.value),
                        OCELNodeRef::Object(ob) => ob
                            .attributes
                            .iter()
                            .filter(|a| &a.name == attr_name.as_ref())
                            .next()
                            .map(|a| &a.value),
                    }
                    .unwrap_or(&OCELAttributeValue::Null);
                    let cel_val = match attr_val {
                        OCELAttributeValue::Float(f) => (*f).into(),
                        OCELAttributeValue::Integer(i) => (*i).into(),
                        OCELAttributeValue::String(s) => s.clone().into(),
                        OCELAttributeValue::Time(t) => t.fixed_offset().into(),
                        OCELAttributeValue::Boolean(b) => (*b).into(),
                        OCELAttributeValue::Null => Value::Null,
                    };
                    Ok(cel_val)
                }

                None => ftx.error("Event or Object not found.").into(),
            };
            res
        },
    );

    context.add_function(
        "attr_at",
        move |ftx: &FunctionContext,
              This(variable): This<Arc<String>>,
              attr_name: Arc<String>,
              at: DateTime<FixedOffset>|
              -> ResolveResult {
            let val = unsafe { var_string_to_val_raw(&variable, binding_raw, ocel_raw) };
            let res = match val {
                Some(val_ref) => {
                    let attr_val = match val_ref {
                        OCELNodeRef::Event(ev) => ev
                            .attributes
                            .iter()
                            .filter(|a| &a.name == attr_name.as_ref())
                            .next()
                            .map(|a| &a.value),
                        OCELNodeRef::Object(ob) => ob
                            .attributes
                            .iter()
                            .filter(|a| &a.name == attr_name.as_ref())
                            .sorted_by_key(|a| a.time)
                            .filter(|a| a.time <= at)
                            .last()
                            .map(|a| &a.value),
                    }
                    .unwrap_or(&OCELAttributeValue::Null);
                    let cel_val = match attr_val {
                        OCELAttributeValue::Float(f) => (*f).into(),
                        OCELAttributeValue::Integer(i) => (*i).into(),
                        OCELAttributeValue::String(s) => s.clone().into(),
                        OCELAttributeValue::Time(t) => t.fixed_offset().into(),
                        OCELAttributeValue::Boolean(b) => (*b).into(),
                        OCELAttributeValue::Null => Value::Null,
                    };
                    Ok(cel_val)
                }

                None => ftx.error("Event or Object not found.").into(),
            };
            res
        },
    );

    context.add_function(
        "time",
        move |ftx: &FunctionContext, This(variable): This<Arc<String>>| -> ResolveResult {
            let val = unsafe { var_string_to_val_raw(&variable, binding_raw, ocel_raw) };
            match val {
                Some(val_ref) => match val_ref {
                    OCELNodeRef::Event(ev) => Ok(ev.time.fixed_offset().into()),
                    _ => ftx.error("Event not found.").into(),
                },

                None => ftx.error("Event not found.").into(),
            }
        },
    );

    // println!("Functions added: {:?}", now.elapsed());

    let res_1 = p.execute(&context).unwrap();

    // println!("Executed : {:?}", now.elapsed());
    unsafe {
        let _ocel_box = Box::from_raw(ocel_raw.0);
    }
    unsafe {
        let _binding_box = Box::from_raw(binding_raw.0);
    }
    match res_1 {
        Value::Bool(b) => b,
        _ => false,
    }
}

pub fn get_vars_in_cel_program(cel: &str) -> HashSet<Variable> {
    lazy_compile_and_insert_into_cache(cel);
    let r_lock = CEL_PROGRAM_CACHE.read().unwrap();
    let p = r_lock.get(cel).unwrap();
    p.references()
        .variables()
        .into_iter()
        .map(|v| string_to_var(v))
        .collect()
}
