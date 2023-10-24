use axum::Json;
use serde_json::{json, Value};

pub async fn root() -> String {
    "root".to_owned()
}
pub async fn get_foo() -> String {
    "get_foo".to_owned()
}
pub async fn post_foo() -> String {
    "post_foo".to_owned()
}
pub async fn foo_bar() -> String {
    "foo_bar".to_owned()
}

pub async fn plain_text() -> String {
    "foo_bar".to_owned()
}

pub async fn json() -> Json<Value> {
    Json(json!({"data":"42"}))
}
