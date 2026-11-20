//! モデルを書く操作の入出力（modelith の ADR-0024 の 1「意図を渡す」・4）。
//!
//! 書くのは 2 段階で行う。
//! 1. 提案: 呼ぶ側が [`EditRequest`] を 1 つ渡し、何も書かずに差分（[`EditProposal`]）を受け取る。
//!    続けて意図を当てたいときは、呼ぶ側が提案を続ければよい。
//! 2. 書き込み: 提案の [`FileChange`] を書き込む。`baseVersion` のあとにファイルが変わっていたら、書き込む側が断る。
//!
//! 要素は修飾名（[`crate::ElementSummary`] の `qualifiedName`）で指す。当てられない意図（指す要素が無い、
//! 入れる先が要素を受けないなど）は、型にせず、呼ぶ側への失敗として返す。
//! 受け手は知らない項目を捨てる。

use serde::{Deserialize, Serialize};

use crate::{ProblemReport, TextEdit};

/// 編集の意図。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum EditRequest {
    /// 要素を足す。
    AddElement {
        /// 要素の種類（例: `part def`）。[`crate::ElementSummary`] の `kind` と同じ言葉。
        kind: String,
        name: String,
        /// 入れる先の要素の修飾名（package に限らない。受けない先は、書く側が断る）。
        parent: String,
    },
    /// 関係を足す。
    AddRelation {
        /// 関係の種類。[`crate::ElementRelation`] の `kind` と同じ言葉。
        kind: String,
        /// 関係が出る側の要素の修飾名（[`crate::RelationDirection::Outgoing`] の側）。
        from: String,
        /// 関係が入る側の要素の修飾名。
        to: String,
    },
    /// 要素の名前を変える。
    Rename { element: String, name: String },
    /// 要素を消す（1 要素ずつ）。
    Delete { element: String },
    /// 要素の説明（`doc`）を書き換える。空の文字列なら説明を消す。
    SetDoc { element: String, doc: String },
    /// ファイルを丸ごと書き換える（テキストで書くほうが向く場面のため）。
    ReplaceText {
        /// プロジェクトのフォルダからのパス（`/` 区切り）。
        file: String,
        text: String,
    },
}

/// 提案の結果。何も書かずに返す。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct EditProposal {
    /// ファイルごとの差分。
    #[serde(default)]
    pub changes: Vec<FileChange>,
    /// 提案を当てたあとの、プロジェクトの問題の一覧。
    #[serde(default)]
    pub problems: Vec<ProblemReport>,
}

/// 1 つのファイルへの差分。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    /// プロジェクトのフォルダからのパス（`/` 区切り）。
    pub file: String,
    /// `edits` が前提にするテキストの版。中身は決めない文字列で、作った側が書き込むときに今の版と比べる。
    pub base_version: String,
    /// 当てる編集（位置は `baseVersion` のテキストのもの）。
    pub edits: Vec<TextEdit>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // --- 正常系 ---

    #[test]
    fn requests_use_type_tag_and_camel_case() {
        // Arrange
        let json = json!({ "type": "addRelation", "kind": "satisfy", "from": "Shop::Cart", "to": "Shop::R1" });
        // Act
        let request: EditRequest = serde_json::from_value(json).unwrap();
        // Assert
        assert_eq!(
            request,
            EditRequest::AddRelation {
                kind: "satisfy".into(),
                from: "Shop::Cart".into(),
                to: "Shop::R1".into()
            }
        );
    }

    #[test]
    fn set_doc_accepts_an_empty_doc() {
        // Arrange（境界値: 空の説明は「説明を消す」）
        let json = json!({ "type": "setDoc", "element": "Shop::Cart", "doc": "" });
        // Act
        let result = serde_json::from_value::<EditRequest>(json);
        // Assert
        assert!(matches!(result, Ok(EditRequest::SetDoc { doc, .. }) if doc.is_empty()));
    }

    #[test]
    fn file_change_uses_base_version() {
        // Arrange
        let json = json!({ "file": "a.sysml", "baseVersion": "v3", "edits": [] });
        // Act
        let change: FileChange = serde_json::from_value(json).unwrap();
        // Assert
        assert_eq!(change.base_version, "v3");
    }

    #[test]
    fn omitted_proposal_lists_are_empty() {
        // Arrange（境界値: 差分も問題も無い提案）
        let json = json!({});
        // Act
        let proposal: EditProposal = serde_json::from_value(json).unwrap();
        // Assert
        assert_eq!(proposal, EditProposal::default());
    }

    #[test]
    fn ignores_unknown_fields() {
        // Arrange（新しい版の送り手が項目を足した場合）
        let json = json!({ "type": "delete", "element": "A", "cascade": true });
        // Act
        let result = serde_json::from_value::<EditRequest>(json);
        // Assert
        assert!(result.is_ok(), "{result:?}");
    }

    // --- 異常系 ---

    #[test]
    fn rejects_an_unknown_type() {
        // Arrange
        let json = json!({ "type": "move", "element": "A" });
        // Act
        let result = serde_json::from_value::<EditRequest>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_change_without_base_version() {
        // Arrange
        let json = json!({ "file": "a.sysml", "edits": [] });
        // Act
        let result = serde_json::from_value::<FileChange>(json);
        // Assert
        assert!(result.is_err());
    }
}
