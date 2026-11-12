<p align="center">
  <img src="https://raw.githubusercontent.com/modelith/.github/main/profile/logo.svg" width="88" height="88" alt="Modelith">
</p>

<h1 align="center">Modelith Plugin SDK</h1>

<p align="center">
  <b><a href="https://github.com/modelith/modelith">Modelith</a> のプラグインを作るための型・JSON Schema・ヘルパー</b><br>
  <sub>Types, JSON Schemas and helpers for building Modelith plugins</sub>
</p>

<p align="center">
  <a href="README.md">English</a> &nbsp;|&nbsp;
  <a href="#はじめる">はじめる</a> &nbsp;|&nbsp;
  <a href="docs/README.ja.md">文書</a> &nbsp;|&nbsp;
  <a href="packages/typescript">TypeScript</a> &nbsp;|&nbsp;
  <a href="crates/modelith-plugin-protocol">Rust</a> &nbsp;|&nbsp;
  <a href="schema">JSON Schema</a> &nbsp;|&nbsp;
  <a href="CONTRIBUTING.md">開発に参加する</a> &nbsp;|&nbsp;
  <a href="https://github.com/sponsors/modelith">支援する</a>
</p>

<p align="center">
  <a href="https://github.com/modelith/plugin-sdk/actions/workflows/ci.yml?query=branch%3Adev"><img src="https://github.com/modelith/plugin-sdk/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI"></a>
  <a href="#ライセンス"><img src="https://img.shields.io/badge/license-MIT-blue" alt="license"></a>
</p>

Modelith は、テキストと図を同期して編集できるオープンソースの SysML v2 モデリングエディタです。
このリポジトリは **Modelith とプラグインの契約** を定めます。プラグインのマニフェスト、検査ルールのファイルの形、
プラグインとホストがやり取りするメッセージ、プラグインレジストリの索引エントリです。
エディタ本体・プラグインの作者・レジストリの CI が、この契約に依存します。

> [!NOTE]
> SDK は 1.0 未満（`apiVersion` は `"0"`）で、マイナーリリースの間でも契約が変わることがあります。
> パッケージはまだ npm・crates.io に公開していません。[インストール](#インストール) をご覧ください。

## プラグインの種類

| `runtime` | 書くもの | 動く場所 | 向いている用途 |
| --- | --- | --- | --- |
| `rules` | YAML のルールファイル | ブラウザ・CLI・CI（Modelith が解釈する） | 「どの要求も何かに満たされている」などのモデルの検査 |
| `browser` | 1 つにまとめたスクリプト | サンドボックス化した iframe の中の Web Worker | インポータ・エクスポータ・ビュー・コマンド・独自の検査 |
| `wasm-component` | WebAssembly Component | ブラウザ・CLI・サーバ | ヘッドレスでも動かしたいプラグイン（予定） |

プラグインは、追加するもの（`contributes`）と必要な権限（`permissions`）をマニフェスト `modelith-plugin.json` に宣言します。
宣言していない操作は Modelith が拒否します。

## はじめる

### rules プラグイン

`modelith-plugin.json`:

```json
{
  "id": "com.example.req-checks",
  "name": "要求の検査",
  "version": "0.1.0",
  "apiVersion": "0",
  "runtime": "rules",
  "license": "MIT",
  "permissions": ["model:read"],
  "contributes": { "rules": ["rules/requirements.yaml"] }
}
```

`rules/requirements.yaml`:

```yaml
rules:
  - id: req-has-satisfier
    severity: warning
    message: "要求「{name}」を満たす要素がありません"
    match:
      kind: requirement def
    require:
      incoming: satisfy
```

書ける条件の全体は [`schema/rule-set.schema.json`](schema/rule-set.schema.json) と、
Modelith の [検査ルールのリファレンス](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md) にあります。

### browser プラグイン

```ts
import { connectToHost, type Diagnostic } from "@modelith/plugin-sdk";

const host = connectToHost(self);

host.onMessage((message) => {
  switch (message.type) {
    case "activate":
      host.send({ type: "ready" });
      break;
    case "didChange": {
      const items: Diagnostic[] = [];
      // message.text や、マニフェストで "model:read" を宣言したときの message.model を調べる
      host.send({ type: "diagnostics", version: message.version, items });
      break;
    }
  }
});
```

Chromium はサンドボックス化した iframe の中でモジュール形式のスクリプトを動かせないので、
1 つの従来型スクリプトにまとめ、`main` で指します。

```sh
npx esbuild src/index.ts --bundle --format=iife --outfile=dist/index.js
```

```json
{ "runtime": "browser", "main": "dist/index.js", "permissions": ["model:read"] }
```

マニフェスト・ルールファイル・メッセージのやり取りの全体の例は [`examples/`](examples) にあります。
順を追って作るときは、手引き（[rules プラグイン](docs/tutorials/rules-plugin.ja.md)・[browser プラグイン](docs/tutorials/browser-plugin.ja.md)）をご覧ください。

## インストール

最初のリリースまでは、Git のリポジトリを使ってください。

```toml
# Rust（Cargo.toml）
[dependencies]
modelith-plugin-protocol = { git = "https://github.com/modelith/plugin-sdk", branch = "dev" }
```

TypeScript は、`packages/typescript` で `npm ci && npm run build` を実行して組み立ててください。
最初のリリースの後は、次のように入れられます。

```sh
npm install @modelith/plugin-sdk
cargo add modelith-plugin-protocol
```

## このリポジトリの構成

| パス | 内容 |
| --- | --- |
| [`crates/modelith-plugin-protocol`](crates/modelith-plugin-protocol) | 契約の唯一の定義元（Rust の型） |
| [`schema/`](schema) | Rust の型から生成した JSON Schema |
| [`packages/typescript`](packages/typescript) | `@modelith/plugin-sdk`（生成した TypeScript の型とヘルパー） |
| [`examples/`](examples) | マニフェスト・ルールファイル・索引エントリ・メッセージのサンプル（Rust とスキーマの両方で検証） |

| スキーマ | 対象 |
| --- | --- |
| [`plugin-manifest`](schema/plugin-manifest.schema.json) | `modelith-plugin.json` |
| [`rule-set`](schema/rule-set.schema.json) | `rules` プラグインのルールファイル |
| [`host-message`](schema/host-message.schema.json) / [`plugin-message`](schema/plugin-message.schema.json) | Modelith と `browser` プラグインの間のメッセージ |
| [`registry-entry`](schema/registry-entry.schema.json) | プラグインレジストリの索引エントリ |

## バージョン

マニフェストの `apiVersion` は SDK のメジャーバージョンです。`"0"` の間は互換性を約束しません。
1.0.0 以降は、削除・改名・必須フィールドの追加にはメジャーバージョンを上げます。
規則の全体は Modelith の [バージョンの規則](https://github.com/modelith/modelith/blob/dev/docs/reference/versioning.md) に、
プラグインのホストの設計は [ADR-0018](https://github.com/modelith/modelith/blob/dev/docs/adr/0018-plugin-host.md) にあります。

## 開発に参加する

不具合の報告や Pull Request を歓迎します。組み立て方・テストの方法・契約の変え方は [CONTRIBUTING.md](CONTRIBUTING.md) にあります。

## ライセンス

MIT です。これらのインタフェースだけを使うプラグインは、どのライセンスでも選べます
（[Modelith Plugin Exception](https://github.com/modelith/modelith/blob/main/PLUGIN-EXCEPTION.md) を参照）。
