# Seedess Rust SDK

Official Rust client for the [Seedess API](https://seedess.com/docs/api/).

> **Status:** planned for `0.1.0`. The API contract is live; this client is next.

## What it covers

- create and refresh streams
- read offload, CDN and P2P metrics
- send per-session telemetry
- manage server-side API keys and per-stream public tokens

## Install (planned)

```bash
cargo add seedess
```

## Contract

Generated from `contracts/api.json` in the Seedess monorepo and checked against the same
JSON Schemas. See the [API reference](https://seedess.com/docs/api/) and
[Libraries](https://seedess.com/docs/libraries/).

## Links

- Docs: https://seedess.com/docs/
- API: https://seedess.com/docs/api/
- Issues: https://github.com/Seedess/rust/issues

## License

MIT.
