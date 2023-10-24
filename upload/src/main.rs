use axum::{
    body::{self, Body},
    extract::{path, ContentLengthLimit, Multipart, Path},
    http::header::{HeaderMap, HeaderName, HeaderValue},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

use rand::prelude::random;
use std::fs::read;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{debug, trace};

const SAVE_FILE_BASE_PATH: &str = "/home/alain/web-server/upload/image";

// 上传表单
async fn show_upload() -> Html<&'static str> {
    Html(std::include_str!("../html/upload.html"))
}

// 上传图片
async fn save_image(
    ContentLengthLimit(mut multipart): ContentLengthLimit<
        Multipart,
        {
            1024 * 1024 * 20 //20M
        },
    >,
) -> Result<(StatusCode, HeaderMap), String> {
    if let Some(file) = multipart.next_field().await.unwrap() {
        //文件类型
        let content_type = file.content_type().unwrap().to_string();
        debug!(content_type);
        //校验是否为图片(出于安全考虑)
        if content_type.starts_with("image/") {
            //根据文件类型生成随机文件名(出于安全考虑)
            let rnd = (random::<f32>() * 1000000000 as f32) as i32;
            //提取"/"的index位置
            let index = content_type
                .find("/")
                .map(|i| i)
                .unwrap_or(usize::max_value());
            //文件扩展名
            let mut ext_name = "xxx";
            if index != usize::max_value() {
                ext_name = &content_type[index + 1..];
            }
            //最终保存在服务器上的文件名
            let save_filename = format!("{}/{}.{}", SAVE_FILE_BASE_PATH, rnd, ext_name);

            //文件内容
            let data = file.bytes().await.unwrap();

            //辅助日志
            println!("filename:{},content_type:{}", save_filename, content_type);

            //保存上传的文件
            tokio::fs::write(&save_filename, &data)
                .await
                .map_err(|err| {
                    dbg!(&err);
                    err.to_string()
                })?;

            //上传成功后，显示上传后的图片
            return redirect(format!("/show_image/{}.{}", rnd, ext_name)).await;
        }
    }

    //正常情况，走不到这里来
    println!("{}", "没有上传文件或文件格式不对");

    //当上传的文件类型不对时，下面的重定向有时候会失败(感觉是axum的bug)
    return redirect(format!("/upload")).await;
}

/**
 * 显示图片
 */
async fn show_image(Path(id): Path<String>) -> (HeaderMap, Vec<u8>) {
    let index = id.find(".").map(|i| i).unwrap_or(usize::max_value());
    //文件扩展名
    let mut ext_name = "xxx";
    if index != usize::max_value() {
        ext_name = &id[index + 1..];
    }
    let content_type = format!("image/{}", ext_name);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_str(&content_type).unwrap(),
    );
    let file_name = format!("{}/{}", SAVE_FILE_BASE_PATH, id);
    (headers, read(&file_name).unwrap())
}

/**
 * 重定向
 */
async fn redirect(path: String) -> Result<(StatusCode, HeaderMap), String> {
    let mut headers = HeaderMap::new();
    //重设LOCATION，跳到新页面
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str(&path).unwrap(),
    );
    //302重定向
    Ok((StatusCode::FOUND, headers))
}

async fn show_image2(Path(id): Path<String>) -> Result<impl IntoResponse, String> {
    let file_path = format!("{}/{}", SAVE_FILE_BASE_PATH, id);
    dbg!(&file_path);
    let data = tokio::fs::read(&file_path).await.map_err(|err| {
        dbg!(&err);
        err.to_string()
    })?;

    // 为下载的图片设置适当的Content-Type
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("image/jpeg"),
    ); // 这里假设所有图片都是JPEG格式，根据实际情况进行调整

    Ok(ImageResponse {
        status: StatusCode::OK,
        headers,
        data,
        path: file_path,
    })
}
async fn download_image(Path(id): Path<String>) -> Result<impl IntoResponse, String> {
    let file_path = format!("{}/{}", SAVE_FILE_BASE_PATH, id);
    dbg!(&file_path);
    let data = tokio::fs::read(&file_path).await.map_err(|err| {
        dbg!(&err);
        err.to_string()
    })?;

    let body = Body::from(data);

    Ok(Response::builder()
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_path),
        )
        .body(body)
        .unwrap())
}
struct ImageResponse {
    status: StatusCode,
    headers: HeaderMap,
    data: Vec<u8>,
    path: String,
}
impl IntoResponse for ImageResponse {
    fn into_response(self) -> Response {
        let body = body::boxed(body::Full::from(self.data));
        Response::builder()
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", self.path),
            )
            .body(body)
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_sse=debug,tower_http=debug")
    }
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // our router
    let app = Router::new()
        .route("/upload", get(show_upload))
        .route("/save_image", post(save_image))
        .route("/show_image/:id", get(show_image))
        .route("/download_image/:id", get(download_image))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // run it with hyper on localhost:3000
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
