use warp::Filter;

// 假设的数据源，为简单起见使用一个静态字符串
static DATA: &str = "Hello, world!";

#[tokio::main]
async fn main() {
    let short_poll = warp::path!("short_poll").map(|| warp::reply::json(&DATA));

    warp::serve(short_poll).run(([127, 0, 0, 1], 1030)).await;
}
