use serde_json::{json, Value};

pub fn success_response(data: Value) -> Value {
    json!({
        "status": "success",
        "data": data
    })
}

pub fn error_response(message: &str) -> Value {
    json!({
        "status": "error",
        "error": message
    })
}
