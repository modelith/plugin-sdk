use std::borrow::Cow;
use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// プラグイン ID の正規表現（逆ドメイン形式、英小文字・数字・`-`、2 区切り以上）。
pub const PLUGIN_ID_PATTERN: &str = r"^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$";

/// 逆ドメイン形式のプラグイン ID（例: `jp.modelith.iso15288-rules`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, TS)]
#[ts(type = "string")]
pub struct PluginId(String);

impl<'de> Deserialize<'de> for PluginId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginIdError(pub String);

impl fmt::Display for PluginIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid plugin id {:?}: expected reverse-domain form like \"com.example.my-plugin\"",
            self.0
        )
    }
}

impl std::error::Error for PluginIdError {}

impl PluginId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(s: &str) -> bool {
        let mut segments = 0;
        for seg in s.split('.') {
            let mut chars = seg.chars();
            let ok = chars.next().is_some_and(|c| c.is_ascii_lowercase())
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if !ok {
                return false;
            }
            segments += 1;
        }
        segments >= 2
    }
}

impl TryFrom<String> for PluginId {
    type Error = PluginIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&s) {
            Ok(Self(s))
        } else {
            Err(PluginIdError(s))
        }
    }
}

impl std::str::FromStr for PluginId {
    type Err = PluginIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl From<PluginId> for String {
    fn from(id: PluginId) -> Self {
        id.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl JsonSchema for PluginId {
    fn schema_name() -> Cow<'static, str> {
        "PluginId".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": PLUGIN_ID_PATTERN,
            "description": "Reverse-domain plugin id, e.g. \"com.example.my-plugin\"."
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 正常系（同値クラス: 逆ドメイン形式） ---

    #[test]
    fn accepts_reverse_domain_ids() {
        // Arrange
        let ids = ["com.example.fmea-check", "jp.modelith.iso15288-rules"];
        // Act
        let results: Vec<bool> = ids
            .iter()
            .map(|id| id.parse::<PluginId>().is_ok())
            .collect();
        // Assert
        assert_eq!(results, [true, true]);
    }

    #[test]
    fn accepts_minimum_two_single_letter_segments() {
        // Arrange（境界値: 最短の 2 区切り・各 1 文字）
        let id = "a.b";
        // Act
        let result = id.parse::<PluginId>();
        // Assert
        assert!(result.is_ok());
    }

    // --- 異常系（同値クラス: 区切り不足・大文字・空の区切り・数字始まり・使えない文字） ---

    #[test]
    fn rejects_malformed_ids() {
        // Arrange
        let ids = [
            "",
            "single",
            "Com.example",
            "com..example",
            "com.example.",
            "1com.x",
            "com.ex_ample",
        ];
        // Act
        let accepted: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| id.parse::<PluginId>().is_ok())
            .collect();
        // Assert
        assert!(accepted.is_empty(), "accepted: {accepted:?}");
    }

    #[test]
    fn deserialize_accepts_valid_id() {
        // Arrange
        let json = r#""com.example.ok""#;
        // Act
        let result = serde_json::from_str::<PluginId>(json);
        // Assert
        assert_eq!(result.unwrap().as_str(), "com.example.ok");
    }

    #[test]
    fn deserialize_rejects_invalid_id() {
        // Arrange
        let json = r#""NotValid""#;
        // Act
        let result = serde_json::from_str::<PluginId>(json);
        // Assert
        assert!(result.is_err());
    }
}
