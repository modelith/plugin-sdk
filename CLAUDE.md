# plugin-sdk — エージェント向け作業ガイド

Modelith とプラグインの**契約**（マニフェスト・レジストリ索引・`TextEdit` などの型、JSON Schema）を定義する MIT のリポジトリ。
本体（[modelith/modelith](https://github.com/modelith/modelith)、AGPL）・プラグイン作者・`plugin-registry` の CI がここに依存する。
設計の背景は本体の ADR-0004（プラグイン構成）と ADR-0006（ライセンス）。

## 単一の定義元

```
crates/modelith-plugin-protocol/src/*.rs   ← 唯一の定義元（Rust の型）
        │ UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated
        ├─▶ schema/*.schema.json                     （生成物）
        └─▶ packages/typescript/src/generated/*.ts   （生成物）
examples/*.json  ← Rust（serde）と TS（JSON Schema）の両方で検証する共通サンプル
```

- 生成物は手で編集しない（フックでブロックされる）。型を変えたら再生成してコミットする。
- 型を変えたら `examples/` も更新し、新しいフィールドの使用例を入れる。
- ID パターンなど両言語で使う定数も Rust 側に置き、生成物として TS へ渡す。

## 互換性ルール（最重要）

- `API_VERSION`（= マニフェストの `apiVersion`）は plugin-sdk のメジャーバージョンと一致させる。
- 既存プラグインを壊す変更（必須フィールドの追加、フィールド・列挙値の削除や改名、意味の変更）は**破壊的変更**。
  コミットに `!` を付け、本体側に ADR を追加してから行う。
- 互換を保つ追加は、任意フィールド（`Option` / `#[serde(default)]`）として入れる。
- マニフェストは `deny_unknown_fields`。未知のフィールドを許す方向への変更も ADR で判断する。

## 完了の定義（Definition of Done）

1. `scripts/check.sh` がグリーン（Rust の fmt / clippy / test〔生成物の最新性を含む〕、TS の型検査・テスト・ビルド）
2. 振る舞いの変更にはテストを追加・更新している
3. トピックブランチ上でコミットし、Conventional Commits 形式のメッセージを付けている

## ブランチとコミット

本体と同じ戦略（[branching-strategy.md](https://github.com/modelith/modelith/blob/dev/docs/development/branching-strategy.md)、ADR-0007）に従う。

- 作業ブランチは `dev` から作り、PR は `dev` に向ける（GitHub の既定ブランチは `main` なので、PR 作成時に base を `dev` と明示する。向け先の誤りは CI の `scripts/check-pr-base.sh` が検出する）。`main` / `dev` へ直接 commit / push しない。ブランチ名は `<type>/<topic>`
- **トピック → `dev` は許可なしでマージしてよい。** 条件は CI（`check` / `conventions`）がグリーンで、未解決のレビューコメントがないこと。方式は Squash
- **`main` へのマージ（PR のマージ、auto-merge の有効化を含む）はメンテナの明示的な許可を得てから行う。**
  `dev` → `main` はマイルストーン（リリース）ごと、方式はマージコミット。許可は PR ごとに取り、過去の許可や他の PR への許可を流用しない
- ステージはパスを明示する（`git add -A` / `.` はフックでブロックされる）。`node_modules`・`target`・ビルド成果物はコミットしない
- コミット / PR タイトル: `<type>(<scope>): <summary>`。scope: `protocol` / `schema` / `ts` / `harness`
- リリースは `dev` → `main` のマージ後、`main` に `vMAJOR.MINOR.PATCH` タグを打つ。crate と npm パッケージのバージョンをそろえる

## コマンド

| 目的 | コマンド |
| --- | --- |
| フルチェック（完了前に必須） | `scripts/check.sh` |
| 高速チェック | `scripts/check.sh --fast` |
| 生成物の再生成 | `UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated` |
| TS のテスト | `npm test --prefix packages/typescript` |

## ハーネス（自動で動くもの）

| タイミング | フック | 役割 |
| --- | --- | --- |
| セッション開始 | `session-start.sh` | rustfmt/clippy と npm 依存を用意し、現在ブランチを通知 |
| Bash 実行前 | `guard-git.sh` | main / dev への push・その上での commit・force-push・一括ステージ・禁止ファイルのコミットをブロック |
| マージ操作前 | `guard-merge.sh` | dev 向けの Squash は許可、main 向けは毎回ユーザーの承認を求め、方式の誤りは拒否 |
| ファイル編集前 | `protect-files.sh` | 生成物・lock ファイルの直接編集をブロック |
| ファイル編集後 | `format-rust.sh` | 編集した `.rs` を rustfmt で整形 |
| 停止前 | `verify-on-stop.sh` | 変更があれば `check.sh --fast` を実行し、失敗なら差し戻し |

フックにブロックされたら、回避策を探さず、メッセージに従って手順を正すこと。
エージェントが同じミスを繰り返したら、ハーネス側に再発防止策を追加する PR（`chore(harness): ...`）を作る。
