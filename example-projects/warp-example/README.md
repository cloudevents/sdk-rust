To run the server:

```console
cargo run
```

To test a POST:

```console
curl -d '{"hello": "world"}' \
  -H'content-type: application/json' \
  -H'ce-specversion: 1.0' \
  -H'ce-id: 1' \
  -H'ce-source: http://cloudevents.io' \
  -H'ce-type: dev.knative.example' \
  http://localhost:3030
```
