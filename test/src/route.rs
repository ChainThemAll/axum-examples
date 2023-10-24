use axum::{routing::get, Router};

use crate::handle::*;
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/plain_test", get(plain_text))
        .route("/json", get(json))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/bar", get(foo_bar))
}
