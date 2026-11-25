//! `plugin-registry` の索引エントリ（1 プラグイン 1 ファイル）と、それを 1 つにまとめた索引。

use serde::{Deserialize, Serialize};

use crate::{PluginId, Runtime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    feature = "codegen",
    schemars(title = "Modelith plugin registry entry")
)]
pub struct RegistryEntry {
    pub id: PluginId,
    pub name: String,
    pub publisher: Publisher,
    /// ソースリポジトリの URL。
    pub repository: String,
    #[cfg_attr(feature = "codegen", schemars(regex(pattern = r"^\d+$")))]
    pub api_version: String,
    pub runtime: Runtime,
    /// SPDX ライセンス式。
    pub license: String,
    pub pricing: Pricing,
    pub versions: Vec<RegistryVersion>,
}

/// 登録簿の全エントリを 1 つにまとめた索引（`index.json`）。アプリはこれを取りに行く。
///
/// 古いアプリが新しい索引を読めるよう、知らないフィールドは無視する。
/// 読めないエントリがあっても、アプリはそのエントリだけを飛ばして残りを使う。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "codegen",
    schemars(title = "Modelith plugin registry index")
)]
pub struct RegistryIndex {
    /// 登録されたプラグイン。ID の順に並べる。
    pub plugins: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Publisher {
    pub name: String,
    /// GitHub のユーザー名または組織名。
    pub github: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "lowercase")]
pub enum Pricing {
    Free,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryVersion {
    pub version: String,
    /// 配布物の URL。
    pub url: String,
    /// 配布物の SHA-256（16 進小文字）。
    #[cfg_attr(feature = "codegen", schemars(regex(pattern = r"^[0-9a-f]{64}$")))]
    pub sha256: String,
    /// 配布物の署名。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub signature: Option<String>,
}
