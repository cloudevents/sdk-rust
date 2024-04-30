Install WASMEdge:

https://wasmedge.org/docs/start/install/

To run the server:
```console
cargo run --target wasm32-wasi
```

To test a GET:

```console
curl -sw '%{http_code}\n' http://localhost:9000/health/readiness
```

To test a POST:

```console
curl -d '{"name": "wasi-womble"}' \
  -H'content-type: application/json' \
  -H'ce-specversion: 1.0' \
  -H'ce-id: 1' \
  -H'ce-source: http://cloudevents.io' \
  -H'ce-type: dev.knative.example' \
  http://localhost:9000
```