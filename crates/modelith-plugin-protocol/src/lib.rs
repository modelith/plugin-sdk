//! Modelith とプラグインの契約。
//!
//! ここで定義した Rust の型が唯一の定義元であり、JSON Schema（`schema/`）と
//! TypeScript の型（`packages/typescript/src/generated/`）はここから生成する。
//! Modelith 本体（ホスト側）とレジストリの CI もこの型・スキーマに依存する。

mod edit;
mod id;
mod manifest;
mod registry;
mod rules;

pub use edit::{Diagnostic, DiagnosticSeverity, Position, Range, TextEdit};
pub use id::{PLUGIN_ID_PATTERN, PluginId, PluginIdError};
pub use manifest::{
    Command, Contributes, Exporter, Importer, PaletteItem, Permission, PluginManifest, Runtime,
    View,
};
pub use registry::{Pricing, Publisher, RegistryEntry, RegistryVersion};
pub use rules::{
    CountRange, ElementMatch, OneOrMany, RULE_ID_PATTERN, Requirement, Rule, RuleId, RuleIdError,
    RuleSet,
};

/// このクレートが定義するプラグイン API のバージョン（SDK の MAJOR と同じ値）。
/// マニフェストの `apiVersion` はこの値と一致しなければならない。
/// SDK が 1.0.0 未満の間は `"0"` で、互換性を約束しない（modelith の docs/reference/versioning.md）。
pub const API_VERSION: &str = "0";

pub mod generate;

#[cfg(test)]
mod tests {
    #[test]
    fn api_version_matches_crate_major() {
        // Arrange（バージョン規則: apiVersion = SDK の MAJOR）
        let major = env!("CARGO_PKG_VERSION_MAJOR");
        // Act
        let api_version = super::API_VERSION;
        // Assert
        assert_eq!(api_version, major);
    }
}
