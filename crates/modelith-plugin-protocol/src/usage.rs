//! 利用状況の送信（modelith の ADR-0026）。アプリ・サーバ・受け口が同じ一覧を使う（ADR-0026 の 4）。
//!
//! 送る本文は `type` で区別する 4 種類。どれも `schemaVersion` に [`USAGE_SCHEMA_VERSION`] を載せる。
//! 値は、決まった語（列挙）・区分（境目を doc に書いた列挙）・上限つきの回数（[`Count`]）のどれかで、自由な文字列は受けない
//! （例外は、形を正規表現で決めた版・年月・ID・コードの場所と、公開の登録簿にあるプラグインの ID）。
//! モデルの中身・要素やファイルの名前・エラーの文・端末に残す ID は、どの本文にも入らない（ADR-0026 の 2）。
//!
//! いつ送るか（`x-modelith-when`）と、どの段階で送るか（`x-modelith-level`: `all` は「すべて送る」のときだけ、
//! `error` は「エラーだけ」でも送る）は、JSON Schema の各本文に注記として出す。
//! 受け手は知らない項目を捨てる（版の違う送り手が項目を足しても、本文全体は拒まない）。

use serde::{Deserialize, Serialize};

use crate::PluginId;

/// 利用状況の本文の形の版。項目を足す・変えるたびに 1 つ上げ、受け手は古い版の本文を見分ける。
pub const USAGE_SCHEMA_VERSION: u32 = 1;

/// [`Count`] の上限（1 回の本文・1 項目あたり）。これを超える値は誤りとして受けない。
pub const COUNT_MAX: u32 = 1_000_000;

/// 送る本文。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(tag = "type", rename_all = "camelCase")]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith usage report"))]
pub enum UsageReport {
    /// ブラウザのアプリを起動したとき。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend("x-modelith-when" = "startup", "x-modelith-level" = "all"))
    )]
    Startup(StartupReport),
    /// ブラウザのアプリの、日に 1 回のまとめ（手元で足し込んだ回数）。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend("x-modelith-when" = "daily", "x-modelith-level" = "all"))
    )]
    Daily(DailyReport),
    /// ブラウザのアプリでエラーが起きたとき（すぐ送る）。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend("x-modelith-when" = "error", "x-modelith-level" = "error"))
    )]
    Error(ErrorReport),
    /// 自分で動かすサーバの、日に 1 回のまとめ。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend("x-modelith-when" = "daily", "x-modelith-level" = "all"))
    )]
    ServerDaily(ServerDailyReport),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct StartupReport {
    pub schema_version: u32,
    pub app: AppInfo,
    pub browser: Browser,
    pub os: Os,
    /// 画面の幅の区分。
    pub screen: ScreenClass,
    /// 画面の表示の言語。
    pub language: Language,
    /// 起動にかかった時間の区分。
    pub startup_time: DurationClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub schema_version: u32,
    pub app: AppInfo,
    /// その日に初めて起きた、使い始めの段階（同じ段階は 1 回だけ）。
    #[serde(default)]
    #[cfg_attr(feature = "codegen", schemars(length(max = 5)))]
    pub onboarding: Vec<OnboardingStep>,
    /// 図の種類ごとの表示の回数。
    #[serde(default)]
    pub diagram_views: DiagramViews,
    /// 操作の回数。
    #[serde(default)]
    pub edits: Edits,
    /// エクスポートの形式ごとの回数。
    #[serde(default)]
    pub exports: Exports,
    /// チェックの実行。
    #[serde(default)]
    pub checks: Checks,
    /// プラグインの利用。
    #[serde(default)]
    pub plugins: PluginUsage,
    /// AI による書き換えの回数。
    #[serde(default)]
    pub ai_rewrites: Count,
    /// 解析にかかった時間の区分ごとの回数（ファイルの大きさの区分ごと）。
    #[serde(default)]
    #[cfg_attr(feature = "codegen", schemars(length(max = 16)))]
    pub parse_times: Vec<ParseTime>,
    /// wasm を読み込めなかった回数。
    #[serde(default)]
    pub wasm_load_failures: Count,
    /// このブラウザで使った週の数の区分（ID を持たずに、続けて使われているかを測る）。
    pub weeks_used: WeeksClass,
    /// このブラウザで初めて使った月（`YYYY-MM`）。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^\d{4}-(0[1-9]|1[0-2])$"))
    )]
    pub first_month: String,
}

/// エラーの報告。エラーの文と利用者の文字は含めず、種類とコードの場所だけを送る。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ErrorReport {
    pub schema_version: u32,
    pub app: AppInfo,
    pub kind: ErrorKind,
    /// スタックトレース（内側から外側へ）。
    #[cfg_attr(feature = "codegen", schemars(length(max = 50)))]
    pub frames: Vec<StackFrame>,
}

/// スタックトレースの 1 段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    /// アプリの組み立て物の中のファイル（例: `assets/index-3f2a.js`）。利用者のファイルの名前は入らない。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^[A-Za-z0-9_][A-Za-z0-9_./-]{0,199}$"))
    )]
    pub module: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ServerDailyReport {
    pub schema_version: u32,
    /// 起動ごとに作るランダムな ID（UUID）。端末やサーバに残す ID ではない。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(
            pattern = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        ))
    )]
    pub instance_id: String,
    /// サーバの版（SemVer）。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$"))
    )]
    pub version: String,
    pub os: Os,
    pub deployment: Deployment,
    pub login: Login,
    /// 利用者の数の区分。
    pub users: AmountClass,
    /// 7 日以内に使った人の数の区分。
    pub active_users_7d: AmountClass,
    /// プロジェクトの数の区分。
    pub projects: AmountClass,
    /// 保存容量の区分。
    pub storage: StorageClass,
    /// プラグインの数の区分。
    pub plugins: AmountClass,
}

/// アプリの版と系列。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// アプリの版（SemVer）。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$"))
    )]
    pub version: String,
    pub channel: Channel,
}

/// 図の種類ごとの回数（アプリの自動の図の id ごと）。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct DiagramViews {
    pub trace: Count,
    pub structure: Count,
    pub control: Count,
    pub ibd: Count,
    pub behavior: Count,
    pub seq: Count,
    pub afv: Count,
    pub stm: Count,
    pub all: Count,
}

/// 操作の回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Edits {
    /// 要素の追加。
    pub add: Count,
    /// 関係の追加。
    pub relate: Count,
    pub rename: Count,
    pub delete: Count,
    pub undo: Count,
}

/// エクスポートの形式ごとの回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Exports {
    pub svg: Count,
    pub png: Count,
    pub jpeg: Count,
    pub drawio: Count,
}

/// チェックの実行の回数と、1 回で見つかった問題の数の区分ごとの回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Checks {
    pub runs: Count,
    /// 問題が 0 件だった回数。
    pub no_problems: Count,
    /// 1〜9 件だった回数。
    pub problems_1_to_9: Count,
    /// 10〜99 件だった回数。
    pub problems_10_to_99: Count,
    /// 100 件以上だった回数。
    pub problems_100_or_more: Count,
}

/// プラグインの利用。公開の登録簿にあるものだけ ID を送り、それ以外は回数だけを `unlisted` に足す。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct PluginUsage {
    #[cfg_attr(feature = "codegen", schemars(length(max = 50)))]
    pub listed: Vec<PluginUse>,
    pub unlisted: Count,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct PluginUse {
    /// 公開の登録簿にあるプラグインの ID。
    pub id: PluginId,
    pub uses: Count,
}

/// 解析の時間の区分ごとの回数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ParseTime {
    pub file_size: SizeClass,
    pub duration: DurationClass,
    pub count: Count,
}

/// 上限つきの回数（0 以上 [`COUNT_MAX`] 以下）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS))]
pub struct Count(u32);

impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::try_from(u32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

impl Count {
    pub fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for Count {
    type Error = String;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        if n <= COUNT_MAX {
            Ok(Self(n))
        } else {
            Err(format!("count {n} exceeds the maximum {COUNT_MAX}"))
        }
    }
}

#[cfg(feature = "codegen")]
impl schemars::JsonSchema for Count {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Count".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "integer",
            "minimum": 0,
            "maximum": COUNT_MAX,
            "description": "A count from 0 up to the maximum."
        })
    }
}

/// アプリの公開の系列（modelith の ADR-0022）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Channel {
    #[default]
    Stable,
    Rc,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Browser {
    Chrome,
    Edge,
    Firefox,
    Safari,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Os {
    Windows,
    Macos,
    Linux,
    Android,
    Ios,
    Other,
}

/// 画面の幅の区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum ScreenClass {
    /// 600px 未満。
    Phone,
    /// 600px 以上 1024px 未満。
    Tablet,
    /// 1024px 以上。
    Desktop,
}

/// 画面の表示の言語。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Ja,
    En,
    Other,
}

/// 使い始めの段階。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum OnboardingStep {
    SampleOpened,
    TextWritten,
    DiagramOpened,
    Saved,
    FolderOpened,
}

/// かかった時間の区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum DurationClass {
    /// 1 秒未満。
    #[serde(rename = "lt1s")]
    Under1s,
    /// 1 秒以上 3 秒未満。
    #[serde(rename = "1to3s")]
    From1To3s,
    /// 3 秒以上 10 秒未満。
    #[serde(rename = "3to10s")]
    From3To10s,
    /// 10 秒以上。
    #[serde(rename = "ge10s")]
    Over10s,
}

/// ファイルの大きさの区分（UTF-8 のバイト数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum SizeClass {
    /// 10 KB 未満。
    #[serde(rename = "lt10kb")]
    Under10Kb,
    /// 10 KB 以上 100 KB 未満。
    #[serde(rename = "10to100kb")]
    From10To100Kb,
    /// 100 KB 以上 1 MB 未満。
    #[serde(rename = "100kbTo1mb")]
    From100KbTo1Mb,
    /// 1 MB 以上。
    #[serde(rename = "ge1mb")]
    Over1Mb,
}

/// このブラウザで使った週の数の区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum WeeksClass {
    /// 1 週。
    #[default]
    #[serde(rename = "1")]
    One,
    /// 2〜4 週。
    #[serde(rename = "2to4")]
    From2To4,
    /// 5〜12 週。
    #[serde(rename = "5to12")]
    From5To12,
    /// 13 週以上。
    #[serde(rename = "ge13")]
    Over13,
}

/// 数の区分（サーバの利用者・プロジェクト・プラグインの数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum AmountClass {
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    /// 2〜10。
    #[serde(rename = "2to10")]
    From2To10,
    /// 11〜100。
    #[serde(rename = "11to100")]
    From11To100,
    /// 101〜1000。
    #[serde(rename = "101to1000")]
    From101To1000,
    /// 1001 以上。
    #[serde(rename = "ge1001")]
    Over1000,
}

/// 保存容量の区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum StorageClass {
    /// 100 MB 未満。
    #[serde(rename = "lt100mb")]
    Under100Mb,
    /// 100 MB 以上 1 GB 未満。
    #[serde(rename = "100mbTo1gb")]
    From100MbTo1Gb,
    /// 1 GB 以上 10 GB 未満。
    #[serde(rename = "1to10gb")]
    From1To10Gb,
    /// 10 GB 以上。
    #[serde(rename = "ge10gb")]
    Over10Gb,
}

/// サーバの動かし方。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Deployment {
    Docker,
    Binary,
}

/// サーバのログインの方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Login {
    /// 管理者が作るアカウント。
    Accounts,
    Oidc,
}

/// エラーの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    /// 捕まえられなかった例外。
    UncaughtError,
    /// 処理されなかった Promise の拒否。
    UnhandledRejection,
    /// wasm（コア）の中の panic。
    WasmPanic,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn app() -> serde_json::Value {
        json!({ "version": "0.2.0-beta.5", "channel": "preview" })
    }

    // --- 正常系 ---

    #[test]
    fn startup_report_uses_type_tag_and_camel_case() {
        // Arrange
        let json = json!({
            "type": "startup", "schemaVersion": 1, "app": app(), "browser": "chrome", "os": "macos",
            "screen": "desktop", "language": "ja", "startupTime": "1to3s"
        });
        // Act
        let report: UsageReport = serde_json::from_value(json).unwrap();
        // Assert
        assert!(matches!(
            report,
            UsageReport::Startup(StartupReport {
                startup_time: DurationClass::From1To3s,
                ..
            })
        ));
    }

    #[test]
    fn daily_report_counts_default_to_zero() {
        // Arrange（境界値: 必須の項目だけ。回数の項目はすべて省く）
        let json = json!({
            "type": "daily", "schemaVersion": 1, "app": app(), "weeksUsed": "1", "firstMonth": "2026-11"
        });
        // Act
        let report: UsageReport = serde_json::from_value(json).unwrap();
        // Assert
        let UsageReport::Daily(daily) = report else {
            panic!("not daily")
        };
        assert_eq!(
            (daily.edits, daily.exports.drawio, daily.plugins.unlisted),
            (Edits::default(), Count::default(), Count::default())
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        // Arrange（新しい版の送り手が項目を足した場合）
        let json = json!({
            "type": "error", "schemaVersion": 2, "app": app(), "kind": "wasmPanic", "frames": [], "newField": 1
        });
        // Act
        let result = serde_json::from_value::<UsageReport>(json);
        // Assert
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn accepts_count_at_the_maximum() {
        // Arrange（境界値: 上限ちょうど）
        let n = COUNT_MAX;
        // Act
        let count = serde_json::from_value::<Count>(json!(n));
        // Assert
        assert_eq!(count.unwrap().get(), COUNT_MAX);
    }

    #[test]
    fn class_names_are_the_documented_words() {
        // Arrange
        let values = (
            DurationClass::Under1s,
            SizeClass::From100KbTo1Mb,
            WeeksClass::Over13,
            AmountClass::Zero,
        );
        // Act
        let json = serde_json::to_value(values).unwrap();
        // Assert
        assert_eq!(json, json!(["lt1s", "100kbTo1mb", "ge13", "0"]));
    }

    // --- 異常系 ---

    #[test]
    fn rejects_count_over_the_maximum() {
        // Arrange（境界値: 上限 + 1）
        let n = COUNT_MAX + 1;
        // Act
        let result = serde_json::from_value::<Count>(json!(n));
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_negative_count() {
        // Arrange（境界値: -1）
        let n = -1;
        // Act
        let result = serde_json::from_value::<Count>(json!(n));
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_free_text_in_a_class() {
        // Arrange（区分に決まった語以外を入れる）
        let json = json!({
            "type": "startup", "schemaVersion": 1, "app": app(), "browser": "chrome", "os": "macos",
            "screen": "desktop", "language": "ja", "startupTime": "2.3 seconds"
        });
        // Act
        let result = serde_json::from_value::<UsageReport>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_type() {
        // Arrange
        let json = json!({ "type": "pageView", "schemaVersion": 1 });
        // Act
        let result = serde_json::from_value::<UsageReport>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_an_invalid_plugin_id() {
        // Arrange（登録簿の ID の形でない名前）
        let json = json!({ "id": "My Plugin", "uses": 1 });
        // Act
        let result = serde_json::from_value::<PluginUse>(json);
        // Assert
        assert!(result.is_err());
    }
}
