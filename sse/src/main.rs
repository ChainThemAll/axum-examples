use axum::{
    extract::TypedHeader,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/", get(|| async { "Hello, World!" }));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let mut i = 0;
    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(move || {
        i += 1;
        Event::default().data(format!("hi,{}", &i))
    })
    .map(Ok)
    .throttle(Duration::from_secs(3)); //每3秒，向浏览器发1次消息

    //每隔1秒发1次保活
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
