//! Modelith とプラグインの契約。
//!
//! ここで定義した Rust の型が唯一の定義元であり、JSON Schema（`schema/`）と
//! TypeScript の型（`packages/typescript/src/generated/`）はここから生成する。
//! Modelith 本体（ホスト側）とレジストリの CI もこの型・スキーマに依存する。

mod edit;
mod id;
mod manifest;
mod registry;

pub use edit::{Diagnostic, DiagnosticSeverity, Position, Range, TextEdit};
pub use id::{PLUGIN_ID_PATTERN, PluginId, PluginIdError};
pub use manifest::{
    Command, Contributes, Exporter, Importer, PaletteItem, Permission, PluginManifest, Runtime,
    View,
};
pub use registry::{Pricing, Publisher, RegistryEntry, RegistryVersion};

/// このクレートが定義するプラグイン API のメジャーバージョン。
/// マニフェストの `apiVersion` はこの値と一致しなければならない。
pub const API_VERSION: &str = "1";

pub mod generate;
