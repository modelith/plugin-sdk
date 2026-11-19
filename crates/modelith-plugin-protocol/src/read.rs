//! モデルを読む・確かめる操作の出力（modelith の ADR-0024 の 1・2）。
//!
//! AI のクライアント（MCP）・開いているアプリ・サーバが、同じ形で使う。どれも読むだけの出力で、
//! 受け手は知らない項目を捨てる（版の違う送り手が項目を足しても、全体は拒まない）。
//!
//! - パス（`file`）は、プロジェクトのフォルダからの `/` 区切りの相対パス。
//! - 修飾名（`qualifiedName`）は `::` 区切り（例: `Shop::Cart`）。引用符つきの名前（`'my part'`）も
//!   書かれたまま入るので、形は縛らない。
//! - 位置（`range`）は [`Range`] と同じく 0 始まり、列は UTF-16 のコード単位。

use serde::{Deserialize, Serialize};

use crate::{Diagnostic, ModelPort, Range};

/// プロジェクトの概要。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ProjectOutline {
    #[serde(default)]
    pub files: Vec<ProjectFile>,
    /// 定義と用法（入れ子の要素も含む）。
    #[serde(default)]
    pub elements: Vec<ElementSummary>,
    /// テキストに書いた図（view）。
    #[serde(default)]
    pub diagrams: Vec<DiagramSummary>,
}

/// プロジェクトのファイル。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    /// プロジェクトのフォルダからのパス（`/` 区切り）。
    pub path: String,
}

/// 要素の概要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ElementSummary {
    /// 修飾名（例: `Shop::Cart`）。
    pub qualified_name: String,
    pub name: String,
    /// 短い名前（`<C1>`）。書いていなければ省く。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub short_name: Option<String>,
    /// 要素の種類（例: `part def`）。[`crate::ModelElement`] の `kind` と同じ言葉。
    pub kind: String,
    /// 要素を書いたファイル（[`ProjectFile`] の `path`）。
    pub file: String,
    /// テキスト上の位置。
    pub range: Range,
}

/// 要素の詳細。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ElementDetail {
    #[serde(flatten)]
    pub summary: ElementSummary,
    /// 説明（`doc`）。書いていなければ省く。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub doc: Option<String>,
    /// 型の名前（書かれたまま）。書いていなければ省く。
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub element_type: Option<String>,
    /// 工程（`@Phase`）の名前。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub phase: Option<String>,
    /// 直下の要素（属性・部品・ポートなど）。
    #[serde(default)]
    pub members: Vec<ElementSummary>,
    #[serde(default)]
    pub ports: Vec<ModelPort>,
    /// この要素から出る関係と、この要素に入る関係。
    #[serde(default)]
    pub relations: Vec<ElementRelation>,
}

/// 要素の関係。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ElementRelation {
    /// 関係の種類。今の言葉は次の 13 個（一般の図に出る関係）。
    /// - [`crate::RelationKind`] と同じもの: `satisfy`・`verify`・`frame`・`derivation`・`flow`
    /// - それ以外: `composition`（部品として持つ）・`definedBy`（用法 → その型の定義）・`allocate`・
    ///   `dependency`・`specialize`・`subset`・`redefine`・`reference`
    ///
    /// 送り手が種類を足しても古い受け手が拒まないよう、列挙にせず文字列のままにする。
    /// 受け手は、知らない言葉の関係を「そのほかの関係」として扱う。
    pub kind: String,
    pub direction: RelationDirection,
    /// 相手の要素の修飾名。
    pub other: String,
}

/// 関係の向き（この要素から見て）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum RelationDirection {
    Outgoing,
    Incoming,
}

/// 図（view）の概要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct DiagramSummary {
    pub name: String,
    pub qualified_name: String,
    /// 図の種類。view の型の名前（例: `GeneralView`・`InterconnectionView`）。
    pub kind: String,
    pub file: String,
    pub range: Range,
}

/// 確かめた結果の問題 1 件。問題の種類のコードは [`Diagnostic`] の `code` に入る。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ProblemReport {
    /// 問題のあるファイル。
    pub file: String,
    pub diagnostic: Diagnostic,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn range() -> serde_json::Value {
        json!({ "start": { "line": 1, "character": 4 }, "end": { "line": 1, "character": 12 } })
    }

    // --- 正常系 ---

    #[test]
    fn element_detail_flattens_the_summary() {
        // Arrange
        let json = json!({
            "qualifiedName": "Shop::Cart", "name": "Cart", "kind": "part def", "file": "shop.sysml",
            "range": range(), "type": "Container",
            "relations": [{ "kind": "satisfy", "direction": "outgoing", "other": "Shop::R1" }]
        });
        // Act
        let detail: ElementDetail = serde_json::from_value(json).unwrap();
        // Assert
        assert_eq!(
            (
                detail.summary.qualified_name.as_str(),
                detail.element_type.as_deref(),
                detail.relations[0].direction
            ),
            ("Shop::Cart", Some("Container"), RelationDirection::Outgoing)
        );
    }

    #[test]
    fn omitted_lists_are_empty() {
        // Arrange（境界値: 一覧の項目をすべて省く）
        let json = json!({});
        // Act
        let outline: ProjectOutline = serde_json::from_value(json).unwrap();
        // Assert
        assert_eq!(outline, ProjectOutline::default());
    }

    #[test]
    fn optional_fields_are_omitted_when_absent() {
        // Arrange
        let summary = ElementSummary {
            qualified_name: "A".into(),
            name: "A".into(),
            short_name: None,
            kind: "part def".into(),
            file: "a.sysml".into(),
            range: serde_json::from_value(range()).unwrap(),
        };
        // Act
        let json = serde_json::to_value(summary).unwrap();
        // Assert
        assert!(json.get("shortName").is_none(), "{json}");
    }

    #[test]
    fn ignores_unknown_fields() {
        // Arrange（新しい版の送り手が項目を足した場合）
        let json = json!({ "path": "a.sysml", "size": 10 });
        // Act
        let result = serde_json::from_value::<ProjectFile>(json);
        // Assert
        assert!(result.is_ok(), "{result:?}");
    }

    // --- 異常系 ---

    #[test]
    fn rejects_an_unknown_direction() {
        // Arrange
        let json = json!({ "kind": "satisfy", "direction": "both", "other": "A" });
        // Act
        let result = serde_json::from_value::<ElementRelation>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_problem_without_a_diagnostic() {
        // Arrange
        let json = json!({ "file": "a.sysml" });
        // Act
        let result = serde_json::from_value::<ProblemReport>(json);
        // Assert
        assert!(result.is_err());
    }
}
