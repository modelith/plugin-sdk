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

## License

MIT. Plugins that use only these interfaces may choose any license
(see the [Modelith Plugin Exception](https://github.com/modelith/modelith/blob/main/PLUGIN-EXCEPTION.md)).
