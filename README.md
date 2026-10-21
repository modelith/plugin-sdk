# plugin-sdk

APIs and type definitions for building [Modelith](https://github.com/modelith/modelith) plugins.

| パス | 内容 |
| --- | --- |
| `crates/modelith-plugin-protocol` | 契約の定義元（Rust）。マニフェスト・レジストリ索引・`TextEdit`・`Diagnostic` |
| `schema/` | 生成された JSON Schema（`plugin-manifest`・`registry-entry`） |
| `packages/typescript` | `@modelith/plugin-sdk`（生成された型＋ヘルパー） |
| `examples/` | マニフェストと索引エントリのサンプル |
| `wit/` | WASM Component 用インタフェース（v2 で追加予定） |

## プラグインの種類

| `runtime` | 動く場所 | 用途 |
| --- | --- | --- |
| `rules` | ブラウザ・CLI・CI | YAML で書く検査ルール |
| `browser` | ブラウザ（Web Worker） | インポータ・エクスポータ・ビュー・コマンド |
| `wasm-component` | ブラウザ・CLI・サーバ | ヘッドレスでも動かしたいプラグイン（予定） |

## 依存の追加・更新

lockfile をコミットし、npm の依存は `npm ci` で入れます。インストール時のスクリプトは `.npmrc`（`ignore-scripts=true`）で止め、公開から 7 日たっていない版は入れません（急ぎのセキュリティ修正は除く）。rustup 自身も最新に保ちます（CI はツールチェーンを入れる前に `rustup self update` し、1.29.2 以上を確かめる）。
決まりの全体は Modelith の [依存の追加・更新](https://github.com/modelith/modelith/blob/dev/CONTRIBUTING.md#依存の追加更新) に従います。

## License

MIT. Plugins that use only these interfaces may choose any license
(see the [Modelith Plugin Exception](https://github.com/modelith/modelith/blob/main/PLUGIN-EXCEPTION.md)).
