use std::collections::HashMap;

use axum::{
    extract::{Form, Path, Query, TypedHeader},
    http::header::{HeaderMap, HeaderValue},
    response::Json,
    routing::{get, post},
    Router,
};
use http::StatusCode;
use serde::Deserialize;

// eg: /user/30，将解析出id=30
async fn user_info(Path(id): Path<i32>) -> String {
    format!("user id:{}", id)
}

// eg: /user2/30，将解析出id=30
async fn user_info_2(id: Path<i32>) -> String {
    format!("user id:{}", id.0)
}

// eg: /person/123/30，将解析出id=123, age=30
async fn person(Path((id, age)): Path<(i32, i32)>) -> String {
    format!("id:{},age:{}", id, age)
}

#[derive(Deserialize)]
struct SomeRequest2 {
    a: Option<String>,
    b: Option<i32>,
    c: Option<String>,
    d: Option<u32>,
}

#[derive(Deserialize)]
struct SomeRequest {
    a: String,
    b: i32,
    c: String,
    d: u32,
}

// eg: path_req/a1/b1/c1/d1
async fn path_req(Path(req): Path<SomeRequest>) -> String {
    format!("a:{},b:{},c:{},d:{}", req.a, req.b, req.c, req.d)
}

//eg: query_req/?a=test&b=2&c=abc&d=80
async fn query_req(Query(args): Query<SomeRequest>) -> String {
    format!("a:{},b:{},c:{},d:{}", args.a, args.b, args.c, args.d)
}

//eg: query_req2?a=abc&c=中华人民共和国&d=123
async fn query_req2(Query(args): Query<SomeRequest2>) -> String {
    format!(
        "a:{},b:{},c:{},d:{}",
        args.a.unwrap_or_default(),
        args.b.unwrap_or(-1), //b缺省值指定为-1
        args.c.unwrap_or_default(),
        args.d.unwrap_or_default()
    )
}

//eg: query?a=1&b=1.0&c=xxx
async fn query(Query(params): Query<HashMap<String, String>>) -> String {
    for (key, value) in &params {
        println!("key:{},value:{}", key, value);
    }
    format!("{:?}", params)
}

// 表单提交
async fn form_request(Form(model): Form<SomeRequest2>) -> String {
    format!(
        "a:{},b:{},c:{},d:{}",
        model.a.unwrap_or_default(),
        model.b.unwrap_or(-1), //b缺省值指定为-1
        model.c.unwrap_or_default(),
        model.d.unwrap_or_default()
    )
}

// json提交
async fn json_request(Json(model): Json<SomeRequest>) -> String {
    format!("a:{},b:{},c:{},d:{}", model.a, model.b, model.c, model.d)
}

/**
 * 获取所有请求头
 */
async fn get_all_header(headers: HeaderMap) -> String {
    for (key, value) in &headers {
        println!("key:{:?} , value:{:?}", key, value);
    }
    format!("{:?}", headers)
}

/**
 * 获取http headers中的user_agent头
 */
async fn get_user_agent_header(TypedHeader(user_agent): TypedHeader<headers::UserAgent>) -> String {
    user_agent.to_string()
}

/**
 * 设置cookie并跳转到新页面
 */
async fn set_cookie_and_redirect(mut headers: HeaderMap) -> (StatusCode, HeaderMap, ()) {
    //设置cookie，blog_url为cookie的key
    headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str("blog_url=http://yjmyzz.cnblogs.com/").unwrap(),
    );

    //重设LOCATION，跳到新页面
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str("/get_cookie").unwrap(),
    );
    //302重定向
    (StatusCode::FOUND, headers, ())
}

/**
 * 读取cookie
 */
async fn get_cookie(headers: HeaderMap) -> (StatusCode, String) {
    //读取cookie，并转成字符串
    let cookies = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or("".to_string());

    //cookie空判断
    if cookies.is_empty() {
        println!("cookie is empty!");
        return (StatusCode::OK, "cookie is empty".to_string());
    }

    //将cookie拆成列表
    let cookies: Vec<&str> = cookies.split(';').collect();
    println!("{:?}", cookies);
    for cookie in &cookies {
        //将内容拆分成k=v的格式
        let cookie_pair: Vec<&str> = cookie.split('=').collect();
        if cookie_pair.len() == 2 {
            let cookie_name = cookie_pair[0].trim();
            let cookie_value = cookie_pair[1].trim();
            println!("{:?}", cookie_pair);
            //判断其中是否有刚才设置的blog_url
            if cookie_name == "blog_url" && !cookie_value.is_empty() {
                println!("found:{}", cookie_value);
                return (StatusCode::OK, cookie_value.to_string());
            }
        }
    }
    return (StatusCode::OK, "empty".to_string());
}

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new()
        .route("/user/:id", get(user_info))
        .route("/user2/:id", get(user_info_2))
        .route("/person/:id/:age", get(person))
        .route("/path_req/:a/:b/:c/:d", get(path_req))
        .route("/query_req", get(query_req))
        .route("/query_req2", get(query_req2))
        .route("/query", get(query))
        .route("/form", post(form_request))
        .route("/json", post(json_request))
        .route("/header", get(get_all_header))
        .route("/user_agent", get(get_user_agent_header))
        .route("/set_cookie", get(set_cookie_and_redirect))
        .route("/get_cookie", get(get_cookie));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
