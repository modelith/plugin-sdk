//! `examples/` のサンプルが Rust の型で読めることを確認する（結合テスト: ファイル → serde → 型）。
//! TypeScript 側（packages/typescript）は同じサンプルを JSON Schema で検証する。

#![allow(clippy::unwrap_used)]

use std::path::Path;

use modelith_plugin_protocol::{
    HostMessage, PluginManifest, PluginMessage, RegistryEntry, RuleSet,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

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

#[test]
fn rule_set_parses() {
    // Arrange
    let json = read("rule-set.json");
    // Act
    let result = serde_json::from_str::<RuleSet>(&json);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}

/// 各メッセージが型で読め、書き戻すと元の JSON と同じになること（JSON で往復できる）。
fn round_trip_errors<T: DeserializeOwned + Serialize>(name: &str) -> Vec<String> {
    let messages: Vec<Value> = serde_json::from_str(&read(name)).unwrap();
    messages
        .into_iter()
        .enumerate()
        .filter_map(|(i, original)| {
            let back = serde_json::from_value::<T>(original.clone())
                .and_then(|m| serde_json::to_value(&m));
            match back {
                Ok(v) if v == original => None,
                Ok(v) => Some(format!("{name}[{i}]: round trip changed it to {v}")),
                Err(e) => Some(format!("{name}[{i}]: {e}")),
            }
        })
        .collect()
}

#[test]
fn host_messages_round_trip() {
    // Arrange
    let name = "messages.host.json";
    // Act
    let errors = round_trip_errors::<HostMessage>(name);
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn plugin_messages_round_trip() {
    // Arrange
    let name = "messages.plugin.json";
    // Act
    let errors = round_trip_errors::<PluginMessage>(name);
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}
