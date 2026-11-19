# プラグイン作者向けの文書

[English](README.md)

## 手引き

Modelith のプラグインを初めて書くなら、ここから始めてください。どの手引きも、空のフォルダから Modelith で動くプラグインまでを扱います。

- [はじめての rules プラグイン](tutorials/rules-plugin.ja.md)：YAML のルールでモデルをチェックする（Modelith とコマンドラインで）
- [はじめての browser プラグイン](tutorials/browser-plugin.ja.md)：TypeScript で、チェック・コマンド・エクスポートを追加する

## リファレンス

正確な動きは Modelith の文書に、型とスキーマはこのリポジトリにあります。

- [チェックルール（rules プラグイン）と `modelith check`](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md)
- [TypeScript のプラグイン（browser プラグイン）](https://github.com/modelith/modelith/blob/dev/docs/reference/script-plugins.md)
- [JSON Schema](../schema) と [サンプル](../examples)
- [`@modelith/plugin-sdk`](../packages/typescript)（TypeScript）と [`modelith-plugin-protocol`](../crates/modelith-plugin-protocol)（Rust）
