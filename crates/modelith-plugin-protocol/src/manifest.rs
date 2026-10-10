//! プラグインのマニフェスト（`modelith-plugin.json`）。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::PluginId;

/// プラグインのマニフェスト。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(title = "Modelith plugin manifest")]
pub struct PluginManifest {
    pub id: PluginId,
    /// 表示名。
    pub name: String,
    /// プラグイン自身のバージョン（SemVer）。
    #[schemars(regex(pattern = r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$"))]
    pub version: String,
    /// 対象とするプラグイン API のメジャーバージョン（plugin-sdk のメジャーバージョン）。
    #[schemars(regex(pattern = r"^\d+$"))]
    pub api_version: String,
    pub runtime: Runtime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// SPDX ライセンス式。
    pub license: String,
    /// エントリポイント（`browser`: ESM ファイル、`wasm-component`: .wasm ファイル）。`rules` では不要。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub main: Option<String>,
    /// 要求する権限。宣言していない操作はホストが拒否する。
    #[serde(default)]
    #[ts(as = "Option<Vec<Permission>>", optional)]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    #[ts(as = "Option<Contributes>", optional)]
    pub contributes: Contributes,
}

/// プラグインの実行方式（ADR-0004 の段階に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Runtime {
    /// v0: YAML 宣言ルール。コアが解釈するのでブラウザ・CLI・CI のどこでも動く。
    Rules,
    /// v1: ESM を Web Worker で実行する。ブラウザのみ。
    Browser,
    /// v2: WASM Component。ブラウザ・CLI・サーバで動く。
    WasmComponent,
}

/// プラグインが要求できる権限。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS,
)]
pub enum Permission {
    /// モデル（テキスト・解析結果）を読む。
    #[serde(rename = "model:read")]
    ModelRead,
    /// `TextEdit` による変更案を返す。
    #[serde(rename = "edits:propose")]
    EditsPropose,
    /// ネットワークにアクセスする。
    #[serde(rename = "network")]
    Network,
    /// プラグイン専用の永続ストレージを使う。
    #[serde(rename = "storage")]
    Storage,
}

/// プラグインが追加する機能（拡張点）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contributes {
    /// 検査ルールのファイル（YAML）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub rules: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Importer>>", optional)]
    pub importers: Vec<Importer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Exporter>>", optional)]
    pub exporters: Vec<Exporter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<View>>", optional)]
    pub views: Vec<View>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<PaletteItem>>", optional)]
    pub palette: Vec<PaletteItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Command>>", optional)]
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Importer {
    pub id: String,
    pub label: String,
    /// 対応する拡張子（例: `.reqif`）。
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exporter {
    pub id: String,
    pub label: String,
    /// 出力ファイルの拡張子（例: `.drawio`）。
    pub extension: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct View {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaletteItem {
    pub id: String,
    pub label: String,
    /// 配置時に挿入する SysML v2 テキストの雛形。
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Command {
    pub id: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_rules_manifest() {
        let m: PluginManifest = serde_json::from_str(
            r#"{
                "id": "jp.modelith.iso15288-rules",
                "name": "ISO 15288 フェーズ検査",
                "version": "0.1.0",
                "apiVersion": "1",
                "runtime": "rules",
                "license": "MIT",
                "permissions": ["model:read"],
                "contributes": { "rules": ["rules/phases.yaml"] }
            }"#,
        )
        .unwrap();
        assert_eq!(m.runtime, Runtime::Rules);
        assert_eq!(m.permissions, vec![Permission::ModelRead]);
        assert_eq!(m.contributes.rules, vec!["rules/phases.yaml"]);
    }

    #[test]
    fn rejects_unknown_fields_and_permissions() {
        let base = r#""id":"a.b","name":"x","version":"0.1.0","apiVersion":"1","runtime":"browser","license":"MIT""#;
        assert!(
            serde_json::from_str::<PluginManifest>(&format!("{{{base},\"extra\":1}}")).is_err()
        );
        assert!(
            serde_json::from_str::<PluginManifest>(&format!("{{{base},\"permissions\":[\"fs\"]}}"))
                .is_err()
        );
    }
}
