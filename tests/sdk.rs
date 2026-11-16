use httpmock::prelude::*;
use seedess_sdk::{Client, CreateStreamInput, TelemetryInput};

#[test]
fn create_stream_sends_auth_and_body() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1/streams")
            .header("authorization", "Bearer key_test")
            .json_body(serde_json::json!({
                "url": "https://origin/master.m3u8",
                "id": "demo"
            }));
        then.status(201).json_body(serde_json::json!({
            "ok": true,
            "id": "demo",
            "publicToken": "pk_1",
            "playback": { "signaling": "/s" }
        }));
    });

    let client = Client::with_base_url("key_test", server.base_url()).unwrap();
    let mut input = CreateStreamInput::new("https://origin/master.m3u8");
    input.id = Some("demo".to_string());
    let stream = client.create_stream(input).unwrap();

    assert_eq!(stream.id, "demo");
    assert_eq!(stream.public_token.as_deref(), Some("pk_1"));
    mock.assert();
}

#[test]
fn disable_peers_maps_to_kill() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/api/v1/streams/demo/kill");
        then.status(200).json_body(serde_json::json!({"ok": true}));
    });

    let client = Client::with_base_url("key_test", server.base_url()).unwrap();
    client.disable_peers("demo").unwrap();
    mock.assert();
}

#[test]
fn metrics_parse() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/api/v1/streams/demo/metrics");
        then.status(200).json_body(serde_json::json!({
            "ok": true,
            "id": "demo",
            "metrics": {"cdnBytes": 10, "p2pBytes": 90, "offloadPercent": 90}
        }));
    });

    let client = Client::with_base_url("key_test", server.base_url()).unwrap();
    let metrics = client.get_metrics("demo").unwrap();
    assert_eq!(metrics.p2p_bytes, 90);
    assert_eq!(metrics.offload_percent, 90.0);
}

#[test]
fn telemetry_has_no_api_key() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/api/v1/telemetry");
        then.status(204);
    });

    let client = Client::with_base_url("key_test", server.base_url()).unwrap();
    client.send_telemetry(TelemetryInput::new("s", "demo")).unwrap();
    mock.assert();
}

#[test]
fn rejects_public_token() {
    assert!(Client::new("pk_stream").is_err());
}
