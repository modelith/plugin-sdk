//! プラグインのマニフェスト（`modelith-plugin.json`）。

use serde::{Deserialize, Serialize};

use crate::PluginId;

/// プラグインのマニフェスト。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith plugin manifest"))]
pub struct PluginManifest {
    pub id: PluginId,
    /// 表示名。
    pub name: String,
    /// プラグイン自身のバージョン（SemVer）。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$"))
    )]
    pub version: String,
    /// 対象とするプラグイン API のメジャーバージョン（plugin-sdk のメジャーバージョン）。
    #[cfg_attr(feature = "codegen", schemars(regex(pattern = r"^\d+$")))]
    pub api_version: String,
    pub runtime: Runtime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub description: Option<String>,
    /// 作者の名前。権限を許可する画面などに出す。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub author: Option<String>,
    /// SPDX ライセンス式。
    pub license: String,
    /// 動く本体の版の範囲。書かなければ、どの版の本体でも動かす。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub engines: Option<Engines>,
    /// エントリポイント。`rules` では不要。
    /// - `browser`: 1 つのファイルにまとめた従来型のスクリプト（例: ESM を IIFE にまとめたもの）。
    ///   モジュール形式（`import` / `export` を含むもの）は Chromium のサンドボックス化した iframe の中で動かせない（ADR-0018 §3.1）。
    /// - `wasm-component`: .wasm ファイル。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub main: Option<String>,
    /// 要求する権限。宣言していない操作はホストが拒否する。
    #[serde(default)]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<Permission>>", optional))]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Contributes>", optional))]
    pub contributes: Contributes,
}

/// プラグインが動く本体の版の範囲（本体の ADR-0023 §5）。
///
/// 本体の公開の系列（安定版・リリース候補版・プレビュー版）が同時に動くので、新しい本体の機能に頼るプラグインは、
/// 動く版の範囲をここに書く。範囲の外の本体は、プラグインを動かさずに利用者に知らせる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Engines {
    /// Modelith の版の範囲。比べる記号と版をカンマで区切って並べる（Cargo の版の指定と同じ書き方。例: `">=0.1.0, <0.3.0"`・`"^0.2"`）。
    /// 本体は、自分の版からプレリリースの印（`-beta.1` など）を外して比べる（`0.2.0-beta.1` の本体は `0.2.0` として扱う）。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(
            pattern = r"^\s*(=|>=?|<=?|~|\^)?\d+(\.\d+){0,2}(\s*,\s*(=|>=?|<=?|~|\^)?\d+(\.\d+){0,2})*\s*$"
        ))
    )]
    pub modelith: String,
}

/// プラグインの実行方式（ADR-0004 の段階に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum Runtime {
    /// v0: YAML 宣言ルール。コアが解釈するのでブラウザ・CLI・CI のどこでも動く。
    Rules,
    /// v1: 1 つのファイルにまとめたスクリプトを、サンドボックス化した iframe の中の Web Worker で実行する。ブラウザのみ。
    Browser,
    /// v2: WASM Component。ブラウザ・CLI・サーバで動く。
    WasmComponent,
}

/// プラグインが要求できる権限。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contributes {
    /// 検査ルールのファイル（YAML）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<String>>", optional))]
    pub rules: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<Importer>>", optional))]
    pub importers: Vec<Importer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<Exporter>>", optional))]
    pub exporters: Vec<Exporter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<View>>", optional))]
    pub views: Vec<View>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<PaletteItem>>", optional))]
    pub palette: Vec<PaletteItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "codegen", ts(as = "Option<Vec<Command>>", optional))]
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Importer {
    pub id: String,
    pub label: String,
    /// 対応する拡張子（例: `.reqif`）。
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exporter {
    pub id: String,
    pub label: String,
    /// 出力ファイルの拡張子（例: `.drawio`）。
    pub extension: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct View {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaletteItem {
    pub id: String,
    pub label: String,
    /// 配置時に挿入する SysML v2 テキストの雛形。
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Command {
    pub id: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#""id":"a.b","name":"x","version":"0.1.0","apiVersion":"0","runtime":"browser","license":"MIT""#;

    #[test]
    fn parses_minimal_rules_manifest() {
        // Arrange
        let json = r#"{
            "id": "jp.modelith.iso15288-rules",
            "name": "ISO 15288 フェーズ検査",
            "version": "0.1.0",
            "apiVersion": "0",
            "runtime": "rules",
            "license": "MIT",
            "permissions": ["model:read"],
            "contributes": { "rules": ["rules/phases.yaml"] }
        }"#;
        // Act
        let m: PluginManifest = serde_json::from_str(json).unwrap();
        // Assert
        assert_eq!(
            (m.runtime, m.permissions, m.contributes.rules),
            (
                Runtime::Rules,
                vec![Permission::ModelRead],
                vec!["rules/phases.yaml".to_owned()]
            )
        );
    }

    #[test]
    fn omitted_optional_fields_default_to_empty() {
        // Arrange（境界値: 必須フィールドのみ）
        let json = format!("{{{BASE}}}");
        // Act
        let m: PluginManifest = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(
            (m.permissions.len(), m.contributes, m.main),
            (0, Contributes::default(), None)
        );
    }

    #[test]
    fn parses_author() {
        // Arrange
        let json = format!("{{{BASE},\"author\":\"Example Inc.\"}}");
        // Act
        let m: PluginManifest = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(m.author.as_deref(), Some("Example Inc."));
    }

    #[test]
    fn omitted_author_is_none() {
        // Arrange（境界値: 作者を書かない既存のマニフェスト）
        let json = format!("{{{BASE}}}");
        // Act
        let m: PluginManifest = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(m.author, None);
    }

    #[test]
    fn parses_engines_range() {
        // Arrange
        let json = format!("{{{BASE},\"engines\":{{\"modelith\":\">=0.1.0, <0.3.0\"}}}}");
        // Act
        let m: PluginManifest = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(
            m.engines.map(|e| e.modelith).as_deref(),
            Some(">=0.1.0, <0.3.0")
        );
    }

    #[test]
    fn omitted_engines_is_none() {
        // Arrange（境界値: 範囲を書かない既存のマニフェストは、どの版の本体でも動かす）
        let json = format!("{{{BASE}}}");
        // Act
        let m: PluginManifest = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(m.engines, None);
    }

    #[test]
    fn rejects_engines_without_modelith_or_with_unknown_host() {
        // Arrange（異常系: 本体の名前が無い・知らない本体の名前）
        let missing = format!("{{{BASE},\"engines\":{{}}}}");
        let unknown = format!("{{{BASE},\"engines\":{{\"modelith\":\"^0.2\",\"cli\":\"^1\"}}}}");
        // Act / Assert
        assert!(serde_json::from_str::<PluginManifest>(&missing).is_err());
        assert!(serde_json::from_str::<PluginManifest>(&unknown).is_err());
    }

    #[test]
    fn rejects_unknown_field() {
        // Arrange
        let json = format!("{{{BASE},\"extra\":1}}");
        // Act
        let result = serde_json::from_str::<PluginManifest>(&json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_permission() {
        // Arrange
        let json = format!("{{{BASE},\"permissions\":[\"fs\"]}}");
        // Act
        let result = serde_json::from_str::<PluginManifest>(&json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_required_field() {
        // Arrange（境界値: 必須フィールドが 1 つ欠ける）
        let json =
            r#"{"id":"a.b","name":"x","version":"0.1.0","apiVersion":"0","runtime":"browser"}"#;
        // Act
        let result = serde_json::from_str::<PluginManifest>(json);
        // Assert
        assert!(result.is_err());
    }
}
