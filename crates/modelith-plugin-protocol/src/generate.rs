//! 生成物（JSON Schema・TypeScript 型）の出力。`tests/generated.rs` が最新性を検証する。

use schemars::schema_for;
use ts_rs::{Config, TS};

use crate::*;

/// 生成するファイルの一覧（リポジトリルートからの相対パス, 内容）。
pub fn files() -> Vec<(&'static str, String)> {
    vec![
        (
            "schema/plugin-manifest.schema.json",
            json(schema_for!(PluginManifest)),
        ),
        (
            "schema/registry-entry.schema.json",
            json(schema_for!(RegistryEntry)),
        ),
        (
            "packages/typescript/src/generated/protocol.ts",
            typescript(),
        ),
    ]
}

fn json(schema: schemars::Schema) -> String {
    let mut s = serde_json::to_string_pretty(&schema).unwrap_or_default();
    s.push('\n');
    s
}

fn typescript() -> String {
    let cfg = Config::new();
    let decls = [
        PluginId::decl(&cfg),
        Position::decl(&cfg),
        Range::decl(&cfg),
        TextEdit::decl(&cfg),
        DiagnosticSeverity::decl(&cfg),
        Diagnostic::decl(&cfg),
        Runtime::decl(&cfg),
        Permission::decl(&cfg),
        Importer::decl(&cfg),
        Exporter::decl(&cfg),
        View::decl(&cfg),
        PaletteItem::decl(&cfg),
        Command::decl(&cfg),
        Contributes::decl(&cfg),
        PluginManifest::decl(&cfg),
        Publisher::decl(&cfg),
        Pricing::decl(&cfg),
        RegistryVersion::decl(&cfg),
        RegistryEntry::decl(&cfg),
    ];
    let mut out = String::from(
        "// 生成物。編集しないこと。定義元: crates/modelith-plugin-protocol\n\
         // 再生成: UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated\n\n",
    );
    out.push_str(&format!(
        "export const API_VERSION = \"{API_VERSION}\";\n\
         export const PLUGIN_ID_PATTERN = {PLUGIN_ID_PATTERN:?};\n\n"
    ));
    for d in decls {
        out.push_str("export ");
        out.push_str(&d);
        out.push_str("\n\n");
    }
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}
