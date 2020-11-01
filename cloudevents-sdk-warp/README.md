# CloudEvents SDK Rust - WARP [![Crates badge]][crates.io] [![Docs badge]][docs.rs]

Integration of [CloudEvents SDK](https://github.com/cloudevents/sdk-rust/) with [Warp - Web Server Framework](https://github.com/seanmonstar/warp/).

Look at [CloudEvents SDK README](https://github.com/cloudevents/sdk-rust/) for more info.

Using this crate you can extract CloudEvent from requests and write CloudEvents to responses.

To echo events:

```rust
use cloudevents_sdk_warp::{filter, reply};
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
```

Executing `http` request:

```bash
curl -v \
    -H "ce-specversion: 1.0" \
    -H "ce-id: 2" \
    -H "ce-type: example.event" \
    -H "ce-source: url://example_response/" \
    -H "content-type: application/json" \
    -X POST -d '{ "age": 43, "name": "John Doe", "phones": ["+44 1234567","+44 2345678"] }' \
    http://localhost:3030/
```

Should produce response similar to:

```
* TCP_NODELAY set
* Connected to localhost (127.0.0.1) port 3030 (#0)
> POST / HTTP/1.1
> Host: localhost:3030
> User-Agent: curl/7.64.1
> Accept: */*
> ce-specversion: 1.0
> ce-id: 2
> ce-type: example.event
> ce-source: url://example_response/
> content-type: application/json
> Content-Length: 74
>
* upload completely sent off: 74 out of 74 bytes
< HTTP/1.1 200 OK
< ce-specversion: 1.0
< ce-id: 2
< ce-type: example.event
< ce-source: url://example_response/
< content-type: application/json
< content-length: 74
< date: Mon, 02 Nov 2020 13:33:40 GMT
<
* Connection #0 to host localhost left intact
{ "age": 43, "name": "John Doe", "phones": ["+44 1234567","+44 2345678"] }
```

To create event inside request handlers and send them as responses:

```rust
#[tokio::main]
async fn main() {
    let routes = warp::any().map(|| {
        let event = EventBuilderV10::new()
            .id("1")
            .source(url::Url::parse("url://example_response/").unwrap())
            .ty("example.ce")
            .data(
                mime::APPLICATION_JSON.to_string(),
                json!({
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                        "+44 1234567",
                        "+44 2345678"
                    ]
                }),
            )
            .build();

        match event {
            Ok(event) => Ok(reply::from_event(event)),
            Err(e) => Ok(warp::reply::with_status(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()),
        }
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

[Crates badge]: https://img.shields.io/crates/v/cloudevents-sdk-warp.svg
[crates.io]: https://crates.io/crates/cloudevents-sdk-warp
[Docs badge]: https://docs.rs/cloudevents-sdk-warp/badge.svg
[docs.rs]: https://docs.rs/cloudevents-sdk-warp