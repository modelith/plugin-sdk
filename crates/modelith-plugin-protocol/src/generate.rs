//! 生成物（JSON Schema・TypeScript 型）の出力。`tests/generated.rs` が最新性を検証する。

use schemars::schema_for;
use ts_rs::{Config, TS};

use crate::usage::*;
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
        ("schema/rule-set.schema.json", json(schema_for!(RuleSet))),
        (
            "schema/host-message.schema.json",
            json(schema_for!(HostMessage)),
        ),
        (
            "schema/plugin-message.schema.json",
            json(schema_for!(PluginMessage)),
        ),
        (
            "schema/project-outline.schema.json",
            json(schema_for!(ProjectOutline)),
        ),
        (
            "schema/element-detail.schema.json",
            json(schema_for!(ElementDetail)),
        ),
        (
            "schema/problem-report.schema.json",
            json(schema_for!(ProblemReport)),
        ),
        (
            "schema/usage-report.schema.json",
            json(schema_for!(UsageReport)),
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
        Engines::decl(&cfg),
        PluginManifest::decl(&cfg),
        Publisher::decl(&cfg),
        Pricing::decl(&cfg),
        RegistryVersion::decl(&cfg),
        RegistryEntry::decl(&cfg),
        RuleId::decl(&cfg),
        OneOrMany::decl(&cfg),
        CountRange::decl(&cfg),
        Requirement::decl(&cfg),
        ElementMatch::decl(&cfg),
        Rule::decl(&cfg),
        RuleSet::decl(&cfg),
        PortDirection::decl(&cfg),
        ModelPort::decl(&cfg),
        ModelElement::decl(&cfg),
        RelationKind::decl(&cfg),
        ModelRelation::decl(&cfg),
        ModelSnapshot::decl(&cfg),
        NotifyLevel::decl(&cfg),
        HostMessage::decl(&cfg),
        PluginMessage::decl(&cfg),
        ProjectFile::decl(&cfg),
        ElementSummary::decl(&cfg),
        RelationDirection::decl(&cfg),
        ElementRelation::decl(&cfg),
        ElementDetail::decl(&cfg),
        DiagramSummary::decl(&cfg),
        ProjectOutline::decl(&cfg),
        ProblemReport::decl(&cfg),
        Count::decl(&cfg),
        Channel::decl(&cfg),
        Client::decl(&cfg),
        Sender::decl(&cfg),
        SendLevel::decl(&cfg),
        Browser::decl(&cfg),
        Os::decl(&cfg),
        ScreenClass::decl(&cfg),
        Language::decl(&cfg),
        OnboardingStep::decl(&cfg),
        DurationClass::decl(&cfg),
        SizeClass::decl(&cfg),
        WeeksClass::decl(&cfg),
        AmountClass::decl(&cfg),
        StorageClass::decl(&cfg),
        Deployment::decl(&cfg),
        LoginMethods::decl(&cfg),
        ParseDurationClass::decl(&cfg),
        ErrorName::decl(&cfg),
        ErrorKind::decl(&cfg),
        AutoViews::decl(&cfg),
        CreatedViews::decl(&cfg),
        Edits::decl(&cfg),
        Layout::decl(&cfg),
        Saves::decl(&cfg),
        Exports::decl(&cfg),
        Checks::decl(&cfg),
        ParseTime::decl(&cfg),
        StackFrame::decl(&cfg),
        StartupReport::decl(&cfg),
        DailyReport::decl(&cfg),
        ErrorReport::decl(&cfg),
        ServerDailyReport::decl(&cfg),
        UsageReport::decl(&cfg),
    ];
    let mut out = String::from(
        "// 生成物。編集しないこと。定義元: crates/modelith-plugin-protocol\n\
         // 再生成: UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated\n\n",
    );
    out.push_str(&format!(
        "export const API_VERSION = \"{API_VERSION}\";\n\
         export const PLUGIN_ID_PATTERN = {PLUGIN_ID_PATTERN:?};\n\
         export const RULE_ID_PATTERN = {RULE_ID_PATTERN:?};\n\
         export const USAGE_SCHEMA_VERSION = {USAGE_SCHEMA_VERSION};\n\
         export const COUNT_MAX = {COUNT_MAX};\n\n"
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
