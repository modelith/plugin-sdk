//! `examples/` のサンプルが Rust の型で読めることを確認する（結合テスト: ファイル → serde → 型）。
//! TypeScript 側（packages/typescript）は同じサンプルを JSON Schema で検証する。

#![allow(clippy::unwrap_used)]

use std::path::Path;

use modelith_plugin_protocol::{
    EditProposal, EditRequest, ElementDetail, HostMessage, PluginManifest, PluginMessage,
    ProblemReport, ProjectOutline, RegistryEntry, RuleSet, UsageReport,
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

#[test]
fn usage_reports_round_trip() {
    // Arrange
    let name = "usage-reports.json";
    // Act
    let errors = round_trip_errors::<UsageReport>(name);
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}

/// 1 つの JSON を型で読み、書き戻すと元と同じになるか（JSON で往復できる）。
fn round_trip_one<T: DeserializeOwned + Serialize>(name: &str) -> Result<(), String> {
    let original: Value = serde_json::from_str(&read(name)).unwrap();
    let back = serde_json::from_value::<T>(original.clone())
        .and_then(|m| serde_json::to_value(&m))
        .map_err(|e| format!("{name}: {e}"))?;
    if back == original {
        Ok(())
    } else {
        Err(format!("{name}: round trip changed it to {back}"))
    }
}

#[test]
fn read_outputs_round_trip() {
    // Arrange
    let names = ["read.project-outline.json", "read.element-detail.json"];
    // Act
    let results = [
        round_trip_one::<ProjectOutline>(names[0]),
        round_trip_one::<ElementDetail>(names[1]),
    ];
    // Assert
    let errors: Vec<_> = results.into_iter().filter_map(Result::err).collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn problem_reports_round_trip() {
    // Arrange
    let name = "read.problems.json";
    // Act
    let errors = round_trip_errors::<ProblemReport>(name);
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn write_requests_and_proposal_round_trip() {
    // Arrange
    let names = ["write.requests.json", "write.proposal.json"];
    // Act
    let mut errors = round_trip_errors::<EditRequest>(names[0]);
    errors.extend(round_trip_one::<EditProposal>(names[1]).err());
    // Assert
    assert!(errors.is_empty(), "{errors:?}");
}
