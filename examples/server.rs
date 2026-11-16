//! Full working Seedess example for Rust.
//!
//!   SEEDESS_API_KEY=key_... cargo run --example server
//!   open http://localhost:8787

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use seedess_sdk::{Client, CreateStreamInput};
use serde_json::json;
use std::env;
use tiny_http::{Header, Response, Server, StatusCode};

const ORIGIN: &str = "https://seedess.com";

fn main() {
    let api_key = env::var("SEEDESS_API_KEY").expect("Set SEEDESS_API_KEY (key_...) before running");
    let base_url = env::var("SEEDESS_API_BASE").unwrap_or_else(|_| ORIGIN.to_string());
    let stream_url = env::var("SEEDESS_STREAM_URL")
        .unwrap_or_else(|_| "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string());
    let stream_id = env::var("SEEDESS_STREAM_ID").unwrap_or_else(|_| "sdk-example".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8787".to_string());

    let client = Client::with_base_url(api_key, base_url).expect("valid API key");
    let server = Server::http(format!("127.0.0.1:{port}")).expect("bind");
    println!("Seedess example on http://localhost:{port}");
    println!("The first /api/seedess request creates the stream.");

    let mut cached: Option<serde_json::Value> = None;
    for request in server.incoming_requests() {
        let path = request.url().to_string();
        if path == "/api/seedess" {
            if cached.is_none() {
                match client.create_stream(CreateStreamInput {
                    url: stream_url.clone(),
                    id: Some(stream_id.clone()),
                    title: Some("Seedess Rust SDK example".to_string()),
                    live: None,
                }) {
                    Ok(stream) => {
                        let token = stream.public_token.clone().unwrap_or_default();
                        let playback = stream.playback.unwrap_or_default();
                        cached = Some(json!({
                            "ok": true,
                            "stream": stream.id,
                            "token": token,
                            "playback": {
                                "master": format!("{ORIGIN}{}", playback.master.unwrap_or_default()),
                                "signaling": format!("{ORIGIN}{}", playback.signaling),
                                "config": format!("{ORIGIN}/signaling/config?token={}", utf8_percent_encode(&token, NON_ALPHANUMERIC)),
                            }
                        }));
                    }
                    Err(error) => {
                        let payload = json!({"ok": false, "error": error.to_string()}).to_string();
                        let _ = request.respond(json_response(StatusCode(500), payload));
                        continue;
                    }
                }
            }
            let _ = request.respond(json_response(StatusCode(200), cached.clone().unwrap().to_string()));
            continue;
        }

        if path == "/" || path == "/index.html" {
            let html = include_str!("public/index.html");
            let response = Response::from_string(html).with_header(
                Header::from_bytes("content-type", "text/html; charset=utf-8").unwrap(),
            );
            let _ = request.respond(response);
            continue;
        }

        let _ = request.respond(json_response(
            StatusCode(404),
            json!({"ok": false, "error": "Not found"}).to_string(),
        ));
    }
}

fn json_response(status: StatusCode, body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("content-type", "application/json").unwrap())
}
