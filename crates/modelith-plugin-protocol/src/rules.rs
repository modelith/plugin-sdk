//! 検査ルールのファイル（マニフェストの `contributes.rules` が指す YAML、ADR-0018 §2）。
//!
//! 解釈は modelith-core だけが行う。ここでは形だけを定め、正規表現の妥当性や
//! `min <= max` などの意味の検査は modelith-core が位置つきで報告する。
//! 未知のキーは黙って捨てず、誤りとして返す（すべての構造体が `deny_unknown_fields`）。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::DiagnosticSeverity;

/// ルール ID の正規表現（英小文字・数字・`-`、英小文字か数字で始まる）。
pub const RULE_ID_PATTERN: &str = r"^[a-z0-9][a-z0-9-]*$";

/// 検査ルールのファイル。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith rule set"))]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

/// 1 つの検査ルール。`match` に当たる要素ごとに `require` を確かめ、満たさなければ診断を出す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    /// プラグインの中で一意な ID。診断の `code` になる。
    pub id: RuleId,
    pub severity: DiagnosticSeverity,
    /// 診断のメッセージ。`{name}`（要素名）と `{kind}`（要素の種類）を差し込める。
    pub message: String,
    /// ルールを当てる要素。
    #[serde(rename = "match")]
    pub target: ElementMatch,
    /// 当たった要素が満たすべき条件。
    pub require: Requirement,
}

/// ルールを当てる要素の条件（すべてを満たす要素に当てる）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ElementMatch {
    /// 要素の種類（例: `requirement def`）。並びはいずれかに当たればよい。
    pub kind: OneOrMany,
    /// 工程（`@Phase`）の名前（例: `SystemRequirements`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub phase: Option<String>,
    /// 要素名の正規表現。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub name: Option<String>,
}

/// 当たった要素が満たすべき条件（書いたものすべて）。
///
/// 語彙は任意フィールドの追加で増やす（互換を保つ）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Requirement {
    /// この種類の関係（例: `satisfy`）を 1 本以上受ける。並びはいずれかでよい。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub incoming: Option<OneOrMany>,
    /// この種類の関係（例: `derivation`）を 1 本以上出す。並びはいずれかでよい。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub outgoing: Option<OneOrMany>,
    /// 要素名がこの正規表現に当たる。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub name: Option<String>,
    /// `true` なら工程（`@Phase`）を持つ、`false` なら持たない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub phase: Option<bool>,
    /// ポートの数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub ports: Option<CountRange>,
    /// 同じ形の条件の否定（満たさないこと）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub not: Option<Box<Requirement>>,
}

/// 数の範囲（両端を含む）。省いた側は制限しない。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CountRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub min: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub max: Option<u32>,
}

/// 文字列 1 つか、文字列の並び。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    /// 並びとして見る（1 つなら長さ 1）。
    pub fn as_slice(&self) -> &[String] {
        match self {
            Self::One(s) => std::slice::from_ref(s),
            Self::Many(v) => v,
        }
    }
}

/// ルール ID（例: `req-has-satisfier`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "codegen", ts(type = "string"))]
pub struct RuleId(String);

impl<'de> Deserialize<'de> for RuleId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleIdError(pub String);

impl fmt::Display for RuleIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid rule id {:?}: expected lowercase letters, digits and '-' like \"req-has-satisfier\"",
            self.0
        )
    }
}

impl std::error::Error for RuleIdError {}

impl RuleId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(s: &str) -> bool {
        let is_word = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
        let mut chars = s.chars();
        chars.next().is_some_and(is_word) && chars.all(|c| is_word(c) || c == '-')
    }
}

impl TryFrom<String> for RuleId {
    type Error = RuleIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&s) {
            Ok(Self(s))
        } else {
            Err(RuleIdError(s))
        }
    }
}

impl std::str::FromStr for RuleId {
    type Err = RuleIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl From<RuleId> for String {
    fn from(id: RuleId) -> Self {
        id.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "codegen")]
impl schemars::JsonSchema for RuleId {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RuleId".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": RULE_ID_PATTERN,
            "description": "Rule id unique within the plugin, e.g. \"req-has-satisfier\"."
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_with(require: &str) -> String {
        format!(
            r#"{{"rules":[{{"id":"r","severity":"warning","message":"m","match":{{"kind":"part def"}},"require":{require}}}]}}"#
        )
    }

    fn parse(json: &str) -> Result<RuleSet, serde_json::Error> {
        serde_json::from_str(json)
    }

    // --- 正常系 ---

    #[test]
    fn parses_full_rule() {
        // Arrange
        let json = r#"{"rules":[{
            "id": "req-has-satisfier",
            "severity": "warning",
            "message": "要求「{name}」を満たす部品がありません",
            "match": { "kind": "requirement def", "phase": "SystemRequirements", "name": "^[A-Z]" },
            "require": { "incoming": "satisfy" }
        }]}"#;
        // Act
        let set = parse(json).unwrap();
        // Assert
        assert_eq!(
            set.rules,
            vec![Rule {
                id: "req-has-satisfier".parse().unwrap(),
                severity: DiagnosticSeverity::Warning,
                message: "要求「{name}」を満たす部品がありません".to_owned(),
                target: ElementMatch {
                    kind: OneOrMany::One("requirement def".to_owned()),
                    phase: Some("SystemRequirements".to_owned()),
                    name: Some("^[A-Z]".to_owned()),
                },
                require: Requirement {
                    incoming: Some(OneOrMany::One("satisfy".to_owned())),
                    ..Requirement::default()
                },
            }]
        );
    }

    #[test]
    fn one_or_many_accepts_string_and_list() {
        // Arrange（同値クラス: 文字列 1 つ／並び）
        let inputs = [r#""a""#, r#"["a","b"]"#];
        // Act
        let slices: Vec<Vec<String>> = inputs
            .iter()
            .map(|s| {
                serde_json::from_str::<OneOrMany>(s)
                    .unwrap()
                    .as_slice()
                    .to_vec()
            })
            .collect();
        // Assert
        assert_eq!(slices, [vec!["a"], vec!["a", "b"]]);
    }

    #[test]
    fn parses_every_require_field_and_nested_not() {
        // Arrange
        let json = rule_with(
            r#"{"outgoing":["derivation","flow"],"name":"^[A-Z]\\w*$","phase":true,
                "ports":{"min":1,"max":8},"not":{"incoming":"satisfy"}}"#,
        );
        // Act
        let require = parse(&json).unwrap().rules.remove(0).require;
        // Assert
        assert_eq!(
            require,
            Requirement {
                incoming: None,
                outgoing: Some(OneOrMany::Many(vec![
                    "derivation".to_owned(),
                    "flow".to_owned()
                ])),
                name: Some(r"^[A-Z]\w*$".to_owned()),
                phase: Some(true),
                ports: Some(CountRange {
                    min: Some(1),
                    max: Some(8)
                }),
                not: Some(Box::new(Requirement {
                    incoming: Some(OneOrMany::One("satisfy".to_owned())),
                    ..Requirement::default()
                })),
            }
        );
    }

    #[test]
    fn accepts_empty_rule_list() {
        // Arrange（境界値: ルール 0 件）
        let json = r#"{"rules":[]}"#;
        // Act
        let set = parse(json).unwrap();
        // Assert
        assert!(set.rules.is_empty());
    }

    #[test]
    fn accepts_rule_ids_at_the_edges() {
        // Arrange（境界値: 1 文字・数字始まり・末尾の `-`）
        let ids = ["a", "0", "a-", "req-2"];
        // Act
        let results: Vec<bool> = ids.iter().map(|id| id.parse::<RuleId>().is_ok()).collect();
        // Assert
        assert_eq!(results, [true, true, true, true]);
    }

    // --- 異常系 ---

    #[test]
    fn rejects_malformed_rule_ids() {
        // Arrange（空・`-` 始まり・大文字・使えない文字）
        let ids = ["", "-a", "Req", "req_x", "req.x", "要求"];
        // Act
        let accepted: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| id.parse::<RuleId>().is_ok())
            .collect();
        // Assert
        assert!(accepted.is_empty(), "accepted: {accepted:?}");
    }

    #[test]
    fn rejects_unknown_keys_at_every_level() {
        // Arrange（ルールセット・ルール・match・require・ports・not の各階層）
        let cases = [
            r#"{"rules":[],"extra":1}"#.to_owned(),
            r#"{"rules":[{"id":"r","severity":"hint","message":"m","match":{"kind":"x"},"require":{},"extra":1}]}"#.to_owned(),
            r#"{"rules":[{"id":"r","severity":"hint","message":"m","match":{"kind":"x","extra":1},"require":{}}]}"#.to_owned(),
            rule_with(r#"{"extra":1}"#),
            rule_with(r#"{"ports":{"min":1,"extra":1}}"#),
            rule_with(r#"{"not":{"extra":1}}"#),
        ];
        // Act
        let accepted: Vec<&String> = cases.iter().filter(|c| parse(c).is_ok()).collect();
        // Assert
        assert!(accepted.is_empty(), "accepted: {accepted:?}");
    }

    #[test]
    fn rejects_missing_match_kind() {
        // Arrange（必須の `kind` が欠ける）
        let json = r#"{"rules":[{"id":"r","severity":"hint","message":"m","match":{"phase":"P"},"require":{}}]}"#;
        // Act
        let result = parse(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_severity() {
        // Arrange
        let json = r#"{"rules":[{"id":"r","severity":"fatal","message":"m","match":{"kind":"x"},"require":{}}]}"#;
        // Act
        let result = parse(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_negative_port_count() {
        // Arrange（境界値: min = -1）
        let json = rule_with(r#"{"ports":{"min":-1}}"#);
        // Act
        let result = parse(&json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_kind_that_is_neither_string_nor_list() {
        // Arrange（異常系: 数値・文字列以外を含む並び）
        let inputs = ["1", r#"["a",1]"#];
        // Act
        let accepted: Vec<&str> = inputs
            .iter()
            .copied()
            .filter(|s| serde_json::from_str::<OneOrMany>(s).is_ok())
            .collect();
        // Assert
        assert!(accepted.is_empty(), "accepted: {accepted:?}");
    }
}
