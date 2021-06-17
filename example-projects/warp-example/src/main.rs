use cloudevents::warp::{filter, reply};
use warp::Filter;

#[tokio::main]
async fn main() {
    let routes = warp::any()
        // extracting event from request
        .and(filter::to_event())
        // returning event back
        .map(|event| reply::from_event(event));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
