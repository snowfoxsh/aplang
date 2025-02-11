use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::Value;
use crate::interpreter::FunctionMap;
use crate::{downcast, std_function};

type ApLangMap = HashMap<Value, Value>;

pub(super) fn std_map() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    std_function!(functions => fn MAP() {
        Ok(Value::NativeObject(Rc::new(RefCell::new(ApLangMap::new()))))
    });
    
    std_function!(functions => fn MAP_INSERT(map: Value::NativeObject<ApLangMap>, key: Value, value: Value) {
        downcast!(map => ApLangMap);
        
        Ok(map.insert(key.clone(), value.clone()).unwrap_or(Value::Null))
    });

    std_function!(functions => fn MAP_GET(map: Value::NativeObject<ApLangMap>, key: Value) {
        downcast!(map => ApLangMap);
        
        Ok(map.get(key).cloned().unwrap_or(Value::Null))
    });

    std_function!(functions => fn MAP_CONTAINS_KEY(map: Value::NativeObject<ApLangMap>, key: Value) {
        downcast!(map => ApLangMap);
        
        let maybe = map.contains_key(key);
        
        Ok(Value::Bool(maybe))
    });

    std_function!(functions => fn MAP_VALUES(map: Value::NativeObject<ApLangMap>, key: Value) {
        downcast!(map => ApLangMap);
        
        let values: Vec<Value> = map.values().cloned().collect();
        
        Ok(Value::List(Rc::new(RefCell::new(values))))
    });

    std_function!(functions => fn MAP_KEYS(map: Value::NativeObject<ApLangMap>, key: Value) {
        downcast!(map => ApLangMap);
        
        let values: Vec<Value> = map.keys().cloned().collect();
        
        Ok(Value::List(Rc::new(RefCell::new(values))))
    });

    functions
}
