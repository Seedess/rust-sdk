# seedess-sdk

Official Seedess SDK for Rust.

**Server-side only.** The SDK uses a secret API key (`key_…`) to create and control streams.
Never ship it in a browser or a mobile app. Playback uses the separate `@seedess/*-p2p`
packages and a per-stream public token (`pk_…`).

## Install

```bash
cargo add seedess-sdk
```

## Use

```rust
use seedess_sdk::{Client, CreateStreamInput};

let client = Client::new(std::env::var("SEEDESS_API_KEY")?)?;

let mut input = CreateStreamInput::new("https://origin.example/master.m3u8");
input.id = Some("demo".into());
input.live = Some(true);

let stream = client.create_stream(input)?;
println!("{:?}", stream.public_token);
println!("{:?}", stream.playback.and_then(|p| p.master));
```

Pass only the public token and stream id to the page.

## Full working example

See [`examples/`](./examples) for a server that creates a stream and a browser page
that plays it with hls.js + the Seedess P2P adapter.

## Methods

| Method | HTTP | Purpose |
| --- | --- | --- |
| `create_stream(CreateStreamInput)` | `POST /api/v1/streams` | Ingest a manifest, mint the public token. |
| `get_stream(id)` | `GET /api/v1/streams/:id` | Status, variants, segment count. |
| `list_streams()` | `GET /api/v1/streams` | All streams in the tenant. |
| `refresh_stream(id)` | `POST /api/v1/streams/:id/refresh` | Pull new live segments. |
| `disable_peers(id)` | `POST /api/v1/streams/:id/kill` | CDN only; playback continues. |
| `enable_peers(id)` | `POST /api/v1/streams/:id/revive` | Re-enable peer delivery. |
| `get_metrics(id)` | `GET /api/v1/streams/:id/metrics` | Offload and delivery counters. |
| `send_telemetry(TelemetryInput)` | `POST /api/v1/telemetry` | One session batch (public, rate limited). |

`request(method, path, body, auth)` returns the raw `serde_json::Value`.

## Errors

```rust
match client.create_stream(input) {
    Ok(stream) => println!("{}", stream.id),
    Err(seedess_sdk::Error::Api { status, code, message }) => {
        eprintln!("{status} {code} {message}");
    }
    Err(error) => eprintln!("{error}"),
}
```

## Links

- Docs: https://seedess.com/docs/
- API: https://seedess.com/docs/api/
- Libraries: https://seedess.com/docs/libraries/

## License

MIT.
