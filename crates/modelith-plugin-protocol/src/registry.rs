//! `plugin-registry` の索引エントリ（1 プラグイン 1 ファイル）。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{PluginId, Runtime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(title = "Modelith plugin registry entry")]
pub struct RegistryEntry {
    pub id: PluginId,
    pub name: String,
    pub publisher: Publisher,
    /// ソースリポジトリの URL。
    pub repository: String,
    #[schemars(regex(pattern = r"^\d+$"))]
    pub api_version: String,
    pub runtime: Runtime,
    /// SPDX ライセンス式。
    pub license: String,
    pub pricing: Pricing,
    pub versions: Vec<RegistryVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Publisher {
    pub name: String,
    /// GitHub のユーザー名または組織名。
    pub github: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
pub enum Pricing {
    Free,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryVersion {
    pub version: String,
    /// 配布物の URL。
    pub url: String,
    /// 配布物の SHA-256（16 進小文字）。
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub sha256: String,
    /// 配布物の署名。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub signature: Option<String>,
}
