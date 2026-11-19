//! 利用状況の送信（modelith の ADR-0026）。アプリ・サーバ・受け口が同じ一覧を使う（ADR-0026 の 4）。
//!
//! 送る本文は `kind` で区別する 4 種類で、1 回の要求に 1 つ送る。どれも `schemaVersion` に [`USAGE_SCHEMA_VERSION`] を載せる。
//! 値は、決まった語（列挙）・区分（境目を doc に書いた列挙）・上限つきの回数（[`Count`]）のどれかで、自由な文字列は受けない
//! （例外は、形と長さを正規表現で決めた版・年月・ID・コードの場所と、公開の登録簿にあるプラグインの ID）。
//! モデルの中身・要素やファイルの名前・エラーの文・端末に残す ID・日時は、どの本文にも入らない（ADR-0026 の 2）。
//! 受け口は受け取った日（UTC）で足し込む。
//!
//! JSON Schema の注記:
//! - 各本文: いつ送るか（`x-modelith-when`）、どの段階で送るか（`x-modelith-level`: `all` は「すべて送る」のときだけ、
//!   `error` は「エラーだけ」でも送る）、本文の大きさの上限（`x-modelith-max-bytes`）。
//! - 項目: `x-modelith-dimension: true` の項目（版・系列・送り手）だけを、受け口はほかの項目と掛け合わせて数える。
//!   それ以外の項目は互いに掛け合わせず 1 つずつ数える（組み合わせが端末を見分ける手がかりにならないように）。
//!   掛け合わせて見たい組は、[`ParseTime`] のように 1 つの項目にまとめる。
//!
//! 受け手は知らない項目を捨てる（版の違う送り手が項目を足しても、本文全体は拒まない）。
//! この型は送り手に向けて厳しく、知らない列挙の値も拒む。受け口は、知らない値の項目だけを捨ててもよい。

use serde::{Deserialize, Serialize};

use crate::PluginId;

/// 利用状況の本文の形の版。項目を足す・変えるたびに 1 つ上げ、受け手は古い版の本文を見分ける。
pub const USAGE_SCHEMA_VERSION: u32 = 1;

/// [`Count`] の上限（1 回の本文・1 項目あたり）。送り手はこれで頭打ちにし（[`Count::capped`]）、受け手は超える値を受けない。
pub const COUNT_MAX: u32 = 10_000;

/// 送る本文。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(tag = "kind", rename_all = "camelCase")]
#[cfg_attr(feature = "codegen", schemars(title = "Modelith usage report"))]
pub enum UsageReport {
    /// ブラウザのアプリを起動したとき。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend(
            "x-modelith-when" = "startup",
            "x-modelith-level" = "all",
            "x-modelith-max-bytes" = 1024
        ))
    )]
    Startup(StartupReport),
    /// ブラウザのアプリの、日に 1 回のまとめ（手元で足し込んだ回数。送れたら手元の足し込みを消す）。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend(
            "x-modelith-when" = "daily",
            "x-modelith-level" = "all",
            "x-modelith-max-bytes" = 8192
        ))
    )]
    Daily(Box<DailyReport>),
    /// ブラウザのアプリでエラーが起きたとき（すぐ送る）。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend(
            "x-modelith-when" = "error",
            "x-modelith-level" = "error",
            "x-modelith-max-bytes" = 4096
        ))
    )]
    Error(ErrorReport),
    /// 自分で動かすサーバの、日に 1 回のまとめ。
    #[cfg_attr(
        feature = "codegen",
        schemars(extend(
            "x-modelith-when" = "daily",
            "x-modelith-level" = "all",
            "x-modelith-max-bytes" = 2048
        ))
    )]
    ServerDaily(ServerDailyReport),
}

/// 起動の報告。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct StartupReport {
    pub schema_version: u32,
    #[serde(flatten)]
    pub sender: Sender,
    pub browser: Browser,
    pub os: Os,
    /// 画面の幅の区分。
    pub screen: ScreenClass,
    /// 画面の表示の言語（BCP 47 の言語の部分）。
    pub language: Language,
    /// 手元のフォルダを開けるブラウザか（File System Access）。
    pub folder_access: bool,
    /// 起動にかかった時間の区分。
    pub startup_time: DurationClass,
}

/// 日に 1 回のまとめ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub schema_version: u32,
    #[serde(flatten)]
    pub sender: Sender,
    /// その日に初めて起きた、使い始めの段階（同じ段階は 1 回だけ）。
    #[serde(default)]
    #[cfg_attr(feature = "codegen", schemars(length(max = 5)))]
    pub onboarding: Vec<OnboardingStep>,
    /// 自動の図の種類ごとの表示の回数。
    #[serde(default)]
    pub auto_views: AutoViews,
    /// 作成した図（view）の種類ごとの表示の回数。
    #[serde(default)]
    pub created_views: CreatedViews,
    /// 編集の意図（modelith-core の `Intent` の op）ごとの回数と、元に戻す・やり直す。
    #[serde(default)]
    pub edits: Edits,
    /// 図の見た目だけを変える操作の回数（モデルは変わらない）。
    #[serde(default)]
    pub layout: Layout,
    /// 保存先ごとの保存の回数。
    #[serde(default)]
    pub saves: Saves,
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

/// エラーの報告。エラーの文と利用者の文字は含めず、種類と自分たちのコードの場所だけを送る。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ErrorReport {
    pub schema_version: u32,
    #[serde(flatten)]
    pub sender: Sender,
    /// 送る段階（「すべて送る」と「エラーだけ」の比を数える）。
    pub level: SendLevel,
    pub error_kind: ErrorKind,
    /// スタックトレース（内側から外側へ）。
    #[cfg_attr(feature = "codegen", schemars(length(max = 30)))]
    pub frames: Vec<StackFrame>,
}

/// スタックトレースの 1 段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    /// 配信しているアプリのファイル（例: `assets/index-3f2a.js`）。利用者のファイルの名前は入らない。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^[A-Za-z0-9_][A-Za-z0-9_./-]{0,199}$"))
    )]
    pub module: String,
    /// 関数の名前（wasm の中のコアの関数など、自分たちのコードのもの）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(pattern = r"^[A-Za-z_$][A-Za-z0-9_$:<>.]{0,199}$"))
    )]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub function: Option<String>,
    pub line: u32,
    pub column: u32,
}

/// サーバの日に 1 回のまとめ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ServerDailyReport {
    pub schema_version: u32,
    #[serde(flatten)]
    pub sender: Sender,
    /// 起動ごとに作るランダムな ID（UUID）。サーバに残す ID ではない。
    #[cfg_attr(
        feature = "codegen",
        schemars(regex(
            pattern = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        ))
    )]
    pub instance_id: String,
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

/// 送り手（どの本文にも入る）。受け口はこの 3 つだけを、ほかの項目と掛け合わせて数える。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Sender {
    #[cfg_attr(feature = "codegen", schemars(extend("x-modelith-dimension" = true)))]
    pub client: Client,
    /// 送り手の版（SemVer。例: `0.2.0-rc.1`）。
    #[cfg_attr(
        feature = "codegen",
        schemars(
            regex(pattern = r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$"),
            length(max = 64),
            extend("x-modelith-dimension" = true)
        )
    )]
    pub app_version: String,
    #[cfg_attr(feature = "codegen", schemars(extend("x-modelith-dimension" = true)))]
    pub channel: Channel,
}

/// 送り手のプログラム。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Client {
    /// ブラウザのアプリ。
    Web,
    /// 自分で動かすサーバ。
    Server,
}

/// アプリの公開の系列（modelith の ADR-0022）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Channel {
    Stable,
    Rc,
    Preview,
}

/// 自動の図の種類ごとの回数（アプリの自動の図の id ごと）。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct AutoViews {
    /// トレーサビリティ。
    pub trace: Count,
    /// ブロック定義図。
    pub structure: Count,
    /// 制御ループ。
    pub control: Count,
    /// 内部ブロック図。
    pub ibd: Count,
    /// 振る舞い。
    pub behavior: Count,
    /// シーケンス図。
    pub seq: Count,
    /// アクティビティ図。
    pub afv: Count,
    /// 状態遷移図。
    pub stm: Count,
    /// すべて。
    pub all: Count,
}

/// 作成した図（SysML v2 の view の種類）ごとの回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct CreatedViews {
    /// GeneralView。
    pub general: Count,
    /// InterconnectionView（内部ブロック図）。
    pub interconnection: Count,
    /// ActionFlowView（アクティビティ図）。
    pub action_flow: Count,
    /// StateTransitionView（状態遷移図）。
    pub state_transition: Count,
    /// SequenceView（シーケンス図）。
    pub sequence: Count,
}

/// 編集の回数。名前は modelith-core の `Intent` の op と同じ。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Edits {
    pub add_node: Count,
    pub add_usage: Count,
    pub rename: Count,
    pub delete: Count,
    pub delete_edge: Count,
    pub connect: Count,
    pub add_message: Count,
    pub update_message: Count,
    pub move_message: Count,
    pub set_phase: Count,
    pub init_phases: Count,
    pub make_port: Count,
    pub set_port_dir: Count,
    pub delete_port: Count,
    pub save_view: Count,
    pub add_to_view: Count,
    pub remove_from_view: Count,
    pub rename_view: Count,
    pub delete_view: Count,
    pub set_attribute: Count,
    pub remove_attribute: Count,
    pub set_doc: Count,
    pub undo: Count,
    pub redo: Count,
}

/// 図の見た目だけを変える操作の回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Layout {
    /// 要素の移動。
    pub r#move: Count,
    /// 線の調整。
    pub adjust_edge: Count,
    /// 整列。
    pub align: Count,
}

/// 保存先ごとの回数。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase", default)]
pub struct Saves {
    /// ブラウザの下書き。
    pub draft: Count,
    /// ダウンロード。
    pub download: Count,
    /// 手元のフォルダ（File System Access）。
    pub folder: Count,
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
    /// 画像のコピー。
    pub clipboard: Count,
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

/// 解析の時間の区分ごとの回数（ファイルの大きさ × 時間を 1 つの項目として数える）。
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

impl Count {
    /// 上限で頭打ちにした回数。送り手が手元で足し込んだ数を本文に入れるときに使う。
    pub fn capped(n: u64) -> Self {
        Self(u32::try_from(n.min(u64::from(COUNT_MAX))).unwrap_or(COUNT_MAX))
    }

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

impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::try_from(u32::deserialize(d)?).map_err(serde::de::Error::custom)
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
            "description": "A count from 0 up to the maximum. Senders cap larger counts at the maximum."
        })
    }
}

/// 送る段階（modelith の ADR-0026 の 1）。「送らない」の人の分は届かない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum SendLevel {
    /// すべて送る。
    All,
    /// エラーだけ送る。
    ErrorsOnly,
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

/// 画面の表示の言語（modelith の ADR-0027）。
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(schemars::JsonSchema, ts_rs::TS))]
pub enum WeeksClass {
    /// 1 週。
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

    fn startup() -> serde_json::Value {
        json!({
            "kind": "startup", "schemaVersion": 1, "client": "web", "appVersion": "0.2.0-rc.1",
            "channel": "rc", "browser": "chrome", "os": "macos", "screen": "desktop", "language": "ja",
            "folderAccess": true, "startupTime": "1to3s"
        })
    }

    fn with(mut base: serde_json::Value, key: &str, value: serde_json::Value) -> serde_json::Value {
        base[key] = value;
        base
    }

    // --- 正常系 ---

    #[test]
    fn startup_report_uses_kind_tag_and_flat_sender() {
        // Arrange
        let json = startup();
        // Act
        let report: UsageReport = serde_json::from_value(json).unwrap();
        // Assert
        let UsageReport::Startup(s) = report else {
            panic!("not startup")
        };
        assert_eq!(
            (s.sender.client, s.sender.channel, s.startup_time),
            (Client::Web, Channel::Rc, DurationClass::From1To3s)
        );
    }

    #[test]
    fn daily_report_counts_default_to_zero() {
        // Arrange（境界値: 必須の項目だけ。回数の項目はすべて省く）
        let json = json!({
            "kind": "daily", "schemaVersion": 1, "client": "web", "appVersion": "0.2.0",
            "channel": "stable", "weeksUsed": "1", "firstMonth": "2026-11"
        });
        // Act
        let report: UsageReport = serde_json::from_value(json).unwrap();
        // Assert
        let UsageReport::Daily(daily) = report else {
            panic!("not daily")
        };
        assert_eq!(
            (daily.edits, daily.exports.clipboard, daily.plugins.unlisted),
            (Edits::default(), Count::default(), Count::default())
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        // Arrange（新しい版の送り手が項目を足した場合）
        let json = with(startup(), "newField", json!(1));
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
    fn capped_stops_at_the_maximum() {
        // Arrange（境界値: 上限ちょうど・上限 + 1・u64 の最大）
        let inputs = [u64::from(COUNT_MAX), u64::from(COUNT_MAX) + 1, u64::MAX, 3];
        // Act
        let counts = inputs.map(|n| Count::capped(n).get());
        // Assert
        assert_eq!(counts, [COUNT_MAX, COUNT_MAX, COUNT_MAX, 3]);
    }

    #[test]
    fn class_names_are_the_documented_words() {
        // Arrange
        let values = (
            DurationClass::Under1s,
            SizeClass::From100KbTo1Mb,
            WeeksClass::Over13,
            AmountClass::Zero,
            SendLevel::ErrorsOnly,
        );
        // Act
        let json = serde_json::to_value(values).unwrap();
        // Assert
        assert_eq!(
            json,
            json!(["lt1s", "100kbTo1mb", "ge13", "0", "errorsOnly"])
        );
    }

    #[test]
    fn frame_function_is_omitted_when_absent() {
        // Arrange
        let frame = StackFrame {
            module: "assets/index.js".into(),
            function: None,
            line: 1,
            column: 2,
        };
        // Act
        let json = serde_json::to_value(frame).unwrap();
        // Assert
        assert_eq!(
            json,
            json!({ "module": "assets/index.js", "line": 1, "column": 2 })
        );
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
        let json = with(startup(), "startupTime", json!("2.3 seconds"));
        // Act
        let result = serde_json::from_value::<UsageReport>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_missing_sender_field() {
        // Arrange（必須の送り手の項目 client が無い）
        let mut json = startup();
        json.as_object_mut().unwrap().remove("client");
        // Act
        let result = serde_json::from_value::<UsageReport>(json);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_kind() {
        // Arrange
        let json = with(startup(), "kind", json!("pageView"));
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
