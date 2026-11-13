# はじめての rules プラグイン

[English](rules-plugin.md)

この手引きでは、モデルを 2 つのルールでチェックするプラグインを書き、Modelith で結果を確かめ、同じチェックをコマンドラインからも動かします。
rules プラグインは、マニフェストと YAML のファイルだけでできています。コードもビルドも要りません。

手元のフォルダを開ける Chrome か Edge で、Modelith（<https://modelith.github.io/modelith/>）を使います。
コマンドラインの手順には [Rust](https://rustup.rs/) も要ります。

## 1. モデルのあるプロジェクトを作る

Modelith のプロジェクトは、`modelith.json` のあるフォルダです。次のフォルダを作ります。

```
kettle/
  modelith.json
  model.sysml
```

`kettle/modelith.json` には、プロジェクトの名前と、使うプラグインを書きます。

```json
{
  "name": "Kettle",
  "plugins": {
    "com.example.kettle-checks": { "version": "0.1.0" }
  }
}
```

`kettle/model.sysml` には要求が 2 つあり、満たされているのは 1 つだけです。

```sysml
package Kettle {
    requirement def BoilWater;
    requirement def KeepWarm;
    part def Heater;
    satisfy requirement BoilWater by Heater;
}
```

## 2. プラグインを書く

プロジェクトに追加したプラグインは `.modelith/plugins/<プラグインの ID>/` に置きます。フォルダを作り、2 つのファイルを入れます。

```
kettle/
  .modelith/
    plugins/
      com.example.kettle-checks/
        modelith-plugin.json
        rules.yaml
```

`modelith-plugin.json` はマニフェストです。ID は逆ドメイン形式で、フォルダの名前と同じにします。
`contributes.rules` に、ルールのファイルをプラグインのフォルダからの相対パスで並べます。

```json
{
  "id": "com.example.kettle-checks",
  "name": "Kettle checks",
  "version": "0.1.0",
  "apiVersion": "0",
  "runtime": "rules",
  "license": "MIT",
  "contributes": { "rules": ["rules.yaml"] }
}
```

`rules.yaml` にルールを書きます。各ルールは `match` で要素を選び、選んだ要素のうち `require` を満たさないものを知らせます。

```yaml
rules:
  - id: req-has-satisfier
    severity: warning
    message: "要求「{name}」を満たす要素がありません"
    match:
      kind: requirement def
    require:
      incoming: satisfy
  - id: part-has-ports
    severity: information
    message: "{kind}「{name}」にポートがありません"
    match:
      kind: [part def, part]
    require:
      ports: { min: 1 }
```

- `req-has-satisfier`：どの `requirement def` も、`satisfy` の関係を 1 本以上受けていること。
- `part-has-ports`：どの `part def`・`part` も、ポートを 1 つ以上持つこと。

Node.js があれば、Modelith で開く前にプラグインのフォルダを検査できます。
`@modelith/plugin-sdk` の `modelith-plugin validate` が、マニフェストとルールのファイルの誤り（キーの綴りの誤りなど）を知らせます。

```sh
npx -p https://github.com/modelith/plugin-sdk/releases/download/v0.1.0/modelith-plugin-sdk-0.1.0.tgz modelith-plugin validate kettle/.modelith/plugins/com.example.kettle-checks
```

最初のリリースまでは、[browser プラグインの手引き](browser-plugin.ja.md#1-プロジェクトを用意する) のとおりに組み立て、
代わりに `node plugin-sdk/packages/typescript/dist/bin.js validate …` を実行します。

## 3. Modelith で結果を見る

1. Chrome か Edge で <https://modelith.github.io/modelith/> を開き、`kettle` フォルダを開きます。
2. `model.sysml` を開くと、テキストの下の問題一覧に次の 2 件が出ます。
   - `KeepWarm` に警告：満たす要素がない
   - `Heater` に情報：ポートがない
3. モデルに `satisfy requirement KeepWarm by Heater;` を書き足すと、入力に合わせて警告が消えます。

「プラグイン → プラグインの一覧…」で、プラグイン・ルールの数・マニフェストやルールのエラーを確かめられます。
知らない項目や正しくない正規表現などの誤りは、黙って捨てられることはなく、ファイルと行・列を添えて知らされます。

## 4. コマンドラインでチェックする

`modelith` コマンドも同じルールで動くので、CI でもチェックできます。
リリースに含まれるまでは、ソースから入れます。

```sh
cargo install --git https://github.com/modelith/modelith --branch dev modelith-cli
modelith check kettle
```

手順 1 のモデルでは、結果を 1 件 1 行（パス・行・列・重大度・メッセージ、最後に `[プラグイン ID/ルール ID]`）で出し、最後にまとめを出します。

```
model.sysml:3:5: 警告: 要求「KeepWarm」を満たす要素がありません [com.example.kettle-checks/req-has-satisfier]
model.sysml:4:5: 情報: part def「Heater」にポートがありません [com.example.kettle-checks/part-has-ports]
ファイル 1 件をチェックしました：エラー 0 件・警告 1 件・情報 1 件・ヒント 0 件
```

`error` が 1 件以上あると終了コードは 1、それ以外は 0 です。CI を失敗させたいルールは、`severity` を `error` にします。

## 次に読むもの

- `.modelith/` をプロジェクトと一緒にコミットすれば、プロジェクトを開く人全員と CI が、同じルールを使います。
- 自分のブラウザのどのプロジェクトでも使いたいときは、プラグインのフォルダを別の場所に置き、
  「プラグイン → プラグインの一覧… → 追加…」で追加先に「このブラウザ」を選びます。この方法で追加したプラグインは、コマンドラインからは見えません。
- ルールに書けること（要素の種類・関係の種類・`phase`・`name` の正規表現・`not`）は、
  Modelith の [チェックルールのリファレンス](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md) にあります。
- ルールでは足りないときは、[browser プラグイン](browser-plugin.ja.md) を書きます。
