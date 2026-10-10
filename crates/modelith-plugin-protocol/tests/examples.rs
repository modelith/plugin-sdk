//! `examples/` のサンプルが Rust の型で読めることを確認する。
//! TypeScript 側（packages/typescript）は同じサンプルを JSON Schema で検証する。

#![allow(clippy::unwrap_used)]

use std::path::Path;

use modelith_plugin_protocol::{PluginManifest, RegistryEntry};

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn manifests_parse() {
    for name in ["manifest.rules.json", "manifest.browser.json"] {
        serde_json::from_str::<PluginManifest>(&read(name))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn registry_entry_parses() {
    serde_json::from_str::<RegistryEntry>(&read("registry-entry.json")).unwrap();
}
