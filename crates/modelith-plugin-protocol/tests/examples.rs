//! `examples/` のサンプルが Rust の型で読めることを確認する（結合テスト: ファイル → serde → 型）。
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
    // Arrange
    let names = ["manifest.rules.json", "manifest.browser.json"];
    // Act
    let errors: Vec<String> = names
        .iter()
        .filter_map(|n| {
            serde_json::from_str::<PluginManifest>(&read(n))
                .err()
                .map(|e| format!("{n}: {e}"))
        })
        .collect();
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn registry_entry_parses() {
    // Arrange
    let json = read("registry-entry.json");
    // Act
    let result = serde_json::from_str::<RegistryEntry>(&json);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}
