# modelith-plugin-protocol

The contract between [Modelith](https://github.com/modelith/modelith) and its plugins, as Rust types:
the plugin manifest, rule files, the messages a `browser` plugin exchanges with the host,
text edits and diagnostics, and plugin registry entries.

These types are the single source of truth. The JSON Schemas and the TypeScript types in
[`@modelith/plugin-sdk`](https://github.com/modelith/plugin-sdk/tree/dev/packages/typescript) are generated from them.
Modelith (the host) and the plugin registry's CI depend on this crate.

Pre-1.0 (`apiVersion` `"0"`): the contract may still change between minor releases.

## Example

```rust
use modelith_plugin_protocol::{API_VERSION, Permission, PluginManifest, Runtime};

let manifest: PluginManifest = serde_json::from_str(r#"{
    "id": "com.example.req-checks",
    "name": "Requirement checks",
    "version": "0.1.0",
    "apiVersion": "0",
    "runtime": "rules",
    "license": "MIT",
    "permissions": ["model:read"],
    "contributes": { "rules": ["rules/requirements.yaml"] }
}"#)
.expect("a valid manifest");

assert_eq!(manifest.api_version, API_VERSION);
assert_eq!(manifest.runtime, Runtime::Rules);
assert!(manifest.permissions.contains(&Permission::ModelRead));
```

Manifests and rule files reject unknown fields, so a typo is an error rather than silently ignored.
Messages ignore unknown fields, so a host and a plugin of different versions keep working when either adds an optional field.

## Features

| Feature | Default | What it adds |
| --- | --- | --- |
| `codegen` | off | `schemars::JsonSchema` and `ts_rs::TS` for every type, and the `generate` module that writes the JSON Schemas and TypeScript types |

Without `codegen` the crate depends only on `serde` and `serde_json`.

## License

MIT
