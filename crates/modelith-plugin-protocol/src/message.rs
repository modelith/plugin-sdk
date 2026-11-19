//! v1（`browser`）のプラグインとホストのメッセージ（ADR-0018 §3・§6）。
//!
//! プラグインはサンドボックス化した iframe の中の Worker で動き、ホストと `postMessage` で
//! やり取りする。どのメッセージも `type` で種類を区別する JSON で、バイト列は base64 の文字列にする。
//!
//! `apiVersion` が `"0"` の間は互換性を約束しない（ModelSnapshot には属性値・接続などを足す予定）。
//! 受け手は知らないフィールドを無視する。版の違うホストとプラグインが、任意フィールドの追加で壊れないようにするため
//! （マニフェストやルールのファイルのような、人が書くものとは扱いを変えている）。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Diagnostic, PluginId, Range, TextEdit};

/// ホストからプラグインへのメッセージ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith host message"))]
pub enum HostMessage {
    /// プラグインを起動する。プラグインは `ready` を返す。
    Activate {
        api_version: String,
        plugin_id: PluginId,
        /// プロジェクトの設定（`modelith.json` の `plugins.<id>.settings`）。形はプラグインが決める。無ければ省く。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "codegen", ts(optional, type = "unknown"))]
        settings: Option<Value>,
    },
    /// モデルのテキストが変わった。
    DidChange {
        /// テキストの版。診断や変更案は、どの版に対するものかをこの値で示す。
        version: u32,
        text: String,
        /// 解析結果。マニフェストで `model:read` を宣言したときだけ送る。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "codegen", ts(optional))]
        model: Option<ModelSnapshot>,
    },
    /// マニフェストの `contributes.commands` のコマンドを実行する。
    RunCommand { request_id: u32, command: String },
    /// `contributes.importers` の取り込みを実行する。
    Import {
        request_id: u32,
        importer: String,
        file_name: String,
        /// ファイルの中身（base64）。
        base64: String,
    },
    /// `contributes.exporters` の書き出しを実行する。
    Export {
        request_id: u32,
        exporter: String,
        /// 表示中の図の名前。無ければモデル全体。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "codegen", ts(optional))]
        view: Option<String>,
    },
    /// `storageGet` / `storageSet` への応答。
    StorageResult {
        request_id: u32,
        /// `storageGet` で読んだ値。無ければ省く。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "codegen", ts(optional, type = "unknown"))]
        value: Option<Value>,
    },
}

/// プラグインからホストへのメッセージ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith plugin message"))]
pub enum PluginMessage {
    /// `activate` を受けて準備ができた。
    Ready,
    /// テキストの版 `version` に対する診断。前に送った診断を置き換える。
    Diagnostics {
        version: u32,
        items: Vec<Diagnostic>,
    },
    /// テキストの版 `version` に対する変更案。ホストが検証してから適用する。
    ProposeEdits {
        version: u32,
        edits: Vec<TextEdit>,
        /// 「元に戻す」に出す名前。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "codegen", ts(optional))]
        label: Option<String>,
    },
    /// `runCommand` が終わった。
    CommandDone { request_id: u32 },
    /// `import` の結果（SysML v2 のテキスト）。
    Imported {
        request_id: u32,
        file_name: String,
        text: String,
    },
    /// `export` の結果。
    Exported {
        request_id: u32,
        file_name: String,
        media_type: String,
        /// ファイルの中身（base64）。
        base64: String,
    },
    /// 依頼（`request_id`）に応えられなかった。
    Failed { request_id: u32, message: String },
    /// プラグイン専用のストレージから読む（`storage` 権限が要る）。
    StorageGet { request_id: u32, key: String },
    /// プラグイン専用のストレージに書く（`storage` 権限が要る）。
    StorageSet {
        request_id: u32,
        key: String,
        #[cfg_attr(feature = "codegen", ts(type = "unknown"))]
        value: Value,
    },
    /// 利用者への通知。
    Notify { level: NotifyLevel, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "lowercase")]
pub enum NotifyLevel {
    Info,
    Warning,
    Error,
}

/// `didChange` で送る解析結果。
///
/// `apiVersion` が `"0"` の間は互換性を約束しない（0.2.0 で属性値・接続を足す予定）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ModelSnapshot {
    pub elements: Vec<ModelElement>,
    pub relations: Vec<ModelRelation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ModelElement {
    /// 要素を指す鍵。`ModelRelation` の `from` / `to` はこの値を使う。
    pub key: String,
    pub name: String,
    /// 要素の種類（例: `part def`）。検査ルールの `match.kind` と同じ言葉。
    pub kind: String,
    /// 工程（`@Phase`）の名前。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub phase: Option<String>,
    #[serde(default)]
    pub ports: Vec<ModelPort>,
    /// テキスト上の位置。
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ModelPort {
    pub name: String,
    /// ポートの型の名前。型を書いていなければ省く。
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub port_type: Option<String>,
    /// 向き。書いていなければ省く。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub direction: Option<PortDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
    In,
    Out,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ModelRelation {
    pub id: String,
    pub kind: RelationKind,
    /// 出る側の要素の `key`。
    pub from: String,
    /// 受ける側の要素の `key`。
    pub to: String,
}

/// 関係の種類。検査ルールの `incoming` / `outgoing` と同じ言葉。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    Satisfy,
    Verify,
    Frame,
    Derivation,
    Flow,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // --- 正常系 ---

    #[test]
    fn host_message_uses_type_tag_and_camel_case_fields() {
        // Arrange
        let message = HostMessage::RunCommand {
            request_id: 7,
            command: "fmea.open".to_owned(),
        };
        // Act
        let value = serde_json::to_value(&message).unwrap();
        // Assert
        assert_eq!(
            value,
            json!({ "type": "runCommand", "requestId": 7, "command": "fmea.open" })
        );
    }

    #[test]
    fn unit_variant_is_just_the_tag() {
        // Arrange（境界値: フィールドの無いメッセージ）
        let message = PluginMessage::Ready;
        // Act
        let value = serde_json::to_value(&message).unwrap();
        // Assert
        assert_eq!(value, json!({ "type": "ready" }));
    }

    #[test]
    fn omitted_optional_fields_are_none() {
        // Arrange（境界値: 任意フィールドをすべて省く）
        let json = r#"{"type":"didChange","version":0,"text":""}"#;
        // Act
        let message: HostMessage = serde_json::from_str(json).unwrap();
        // Assert
        assert_eq!(
            message,
            HostMessage::DidChange {
                version: 0,
                text: String::new(),
                model: None
            }
        );
    }

    #[test]
    fn activate_without_settings() {
        // Arrange（設定の無いプロジェクト）
        let json = r#"{"type":"activate","apiVersion":"0","pluginId":"a.b"}"#;
        // Act
        let message: HostMessage = serde_json::from_str(json).unwrap();
        // Assert
        assert!(matches!(
            message,
            HostMessage::Activate { settings: None, .. }
        ));
    }

    #[test]
    fn rejects_invalid_plugin_id_in_activate() {
        // Arrange
        let json = r#"{"type":"activate","apiVersion":"0","pluginId":"Bad"}"#;
        // Act
        let result = serde_json::from_str::<HostMessage>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn ignores_unknown_fields() {
        // Arrange（新しい版の相手が任意フィールドを足した場合）
        let json = r#"{"type":"commandDone","requestId":1,"durationMs":3}"#;
        // Act
        let message: PluginMessage = serde_json::from_str(json).unwrap();
        // Assert
        assert_eq!(message, PluginMessage::CommandDone { request_id: 1 });
    }

    #[test]
    fn port_type_is_named_type_in_json() {
        // Arrange
        let port = ModelPort {
            name: "p".to_owned(),
            port_type: Some("Power".to_owned()),
            direction: Some(PortDirection::Inout),
        };
        // Act
        let value = serde_json::to_value(&port).unwrap();
        // Assert
        assert_eq!(
            value,
            json!({ "name": "p", "type": "Power", "direction": "inout" })
        );
    }

    // --- 異常系 ---

    #[test]
    fn rejects_unknown_type() {
        // Arrange
        let json = r#"{"type":"shutdown"}"#;
        // Act
        let result = serde_json::from_str::<HostMessage>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_type() {
        // Arrange
        let json = r#"{"requestId":1}"#;
        // Act
        let result = serde_json::from_str::<PluginMessage>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_required_field() {
        // Arrange（`failed` に `message` が無い）
        let json = r#"{"type":"failed","requestId":1}"#;
        // Act
        let result = serde_json::from_str::<PluginMessage>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_negative_request_id() {
        // Arrange（境界値: -1）
        let json = r#"{"type":"commandDone","requestId":-1}"#;
        // Act
        let result = serde_json::from_str::<PluginMessage>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_relation_kind() {
        // Arrange（modelith にまだ無い関係）
        let json = r#"{"id":"r1","kind":"refinement","from":"a","to":"b"}"#;
        // Act
        let result = serde_json::from_str::<ModelRelation>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_notify_level() {
        // Arrange
        let json = r#"{"type":"notify","level":"debug","message":"m"}"#;
        // Act
        let result = serde_json::from_str::<PluginMessage>(json);
        // Assert
        assert!(result.is_err());
    }
}
