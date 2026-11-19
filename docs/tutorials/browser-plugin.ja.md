# はじめての browser プラグイン

[English](browser-plugin.md)

この手引きでは、Modelith に次の 3 つを追加する TypeScript のプラグインを書きます。

- チェック：ポートが上限より多い部品に警告を出す（上限はプロジェクトごとに設定できる）
- コマンド：モデルの要素と関係の数を知らせる
- エクスポート：モデルの要素を CSV ファイルに書き出す

その中で、プラグインと Modelith のやり取り、コードのまとめ方、権限の求め方を見ていきます。
チェックだけでよければ、[rules プラグイン](rules-plugin.ja.md) の方が簡単です。

[Node.js](https://nodejs.org/)・Git と、Chrome か Edge で開いた Modelith（<https://modelith.github.io/modelith/>）を使います。

## 1. プロジェクトを用意する

SDK の最初のリリースまでは、このリポジトリから組み立てます。

```sh
git clone https://github.com/modelith/plugin-sdk
npm ci --prefix plugin-sdk/packages/typescript
npm run build --prefix plugin-sdk/packages/typescript
```

その隣に、プラグインのフォルダを作ります。

```sh
mkdir port-check && cd port-check
npm init -y
npm install --save-dev esbuild typescript ../plugin-sdk/packages/typescript
```

最初のリリースの後は、clone と組み立ての代わりに、GitHub Release から SDK を入れます
（npm のレジストリには出していませんが、`@modelith/plugin-sdk` という名前で入ります）。

```sh
npm install --save-dev esbuild typescript https://github.com/modelith/plugin-sdk/releases/download/v0.1.0/modelith-plugin-sdk-0.1.0.tgz
```

次の構成になります。`main.js` は、Modelith が動かす、1 つにまとめたプラグインです。

```
port-check/
  modelith-plugin.json
  package.json
  tsconfig.json
  src/main.ts
  main.js          ← 手順 4 で作る
```

## 2. マニフェストを書く

`modelith-plugin.json` で、プラグインが追加するものと必要な権限を Modelith に伝えます。

```json
{
  "id": "com.example.port-check",
  "name": "ポートのチェック",
  "version": "0.1.0",
  "apiVersion": "0",
  "runtime": "browser",
  "author": "あなたの名前",
  "license": "MIT",
  "main": "main.js",
  "permissions": ["model:read"],
  "contributes": {
    "commands": [{ "id": "summary", "title": "モデルの概要を表示" }],
    "exporters": [{ "id": "elements-csv", "label": "要素の一覧（CSV）", "extension": ".csv" }]
  }
}
```

- `permissions`：`model:read` があると、解析したモデルを受け取り、診断を出せます。
  Modelith はプラグインを動かしてよいかを尋ねるときに権限を示し、宣言していない操作は拒否します。
- `contributes.commands` と `contributes.exporters` は、Modelith の「プラグイン」メニューに出ます。

## 3. プラグインを書く

`tsconfig.json` で、Web Worker のコードとして型を検査させます。

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "WebWorker"],
    "strict": true,
    "noEmit": true
  },
  "include": ["src"]
}
```

`src/main.ts`：

```ts
import { connectToHost, type Diagnostic, type ModelSnapshot } from "@modelith/plugin-sdk";

const host = connectToHost(self);

let version = 0;
let model: ModelSnapshot | undefined;
let maxPorts = 8;

host.onMessage((message) => {
  switch (message.type) {
    case "activate": {
      const settings = message.settings as { maxPorts?: unknown } | undefined;
      if (typeof settings?.maxPorts === "number") maxPorts = settings.maxPorts;
      host.send({ type: "ready" });
      break;
    }
    case "didChange":
      version = message.version;
      model = message.model;
      if (model) host.send({ type: "diagnostics", version, items: checkPorts(model) });
      break;
    case "runCommand":
      if (message.command === "summary") {
        host.send({ type: "notify", level: "info", message: summary(model) });
      }
      host.send({ type: "commandDone", requestId: message.requestId });
      break;
    case "export":
      if (!model) {
        host.send({ type: "failed", requestId: message.requestId, message: "モデルがまだ解析されていません" });
        break;
      }
      host.send({
        type: "exported",
        requestId: message.requestId,
        fileName: "elements.csv",
        mediaType: "text/csv",
        base64: toBase64(toCsv(model)),
      });
      break;
  }
});

function checkPorts(model: ModelSnapshot): Diagnostic[] {
  return model.elements
    .filter((element) => element.ports.length > maxPorts)
    .map((element) => ({
      range: element.range,
      severity: "warning",
      message: `「${element.name}」のポートが ${element.ports.length} 個あります（上限 ${maxPorts} 個）`,
      code: "too-many-ports",
    }));
}

function summary(model: ModelSnapshot | undefined): string {
  if (!model) return "モデルがまだ解析されていません";
  return `要素 ${model.elements.length} 件・関係 ${model.relations.length} 件`;
}

function toCsv(model: ModelSnapshot): string {
  const quote = (value: string) => `"${value.replaceAll('"', '""')}"`;
  const rows = model.elements.map((e) => [e.name, e.kind, e.phase ?? ""].map(quote).join(","));
  return ["name,kind,phase", ...rows].join("\n") + "\n";
}

// btoa() は Latin-1 しか受け付けないので、先に UTF-8 のバイト列にする
function toBase64(text: string): string {
  let binary = "";
  for (const byte of new TextEncoder().encode(text)) binary += String.fromCharCode(byte);
  return btoa(binary);
}
```

やり取りの流れ：

1. Modelith がプロジェクトの設定を添えて `activate` を送り、プラグインは `ready` を返します。
2. Modelith はテキストと、`model:read` があるので解析したモデルを `didChange` で送ります。
   テキストを解析し直すたびに送り直します。プラグインはその `version` に対する `diagnostics` を返します。古い版への診断は使われません。
3. 利用者がコマンドやエクスポートを選ぶと、Modelith は `requestId` を付けて `runCommand` や `export` を送ります。
   プラグインは同じ `requestId` で `commandDone`・`exported`・`failed` のどれかを返します。

メッセージの種類はすべて `HostMessage` と `PluginMessage` の型にあるので、エディタが補完し、誤りを指摘します。

## 4. 1 つのファイルにまとめる

プラグインは、サンドボックス化した iframe の中の Worker で、従来型のスクリプトとして動きます。そこでは `import` で何も読み込めません。
コードと SDK を 1 つのファイルにまとめます。

```sh
npx tsc
npx esbuild src/main.ts --bundle --format=iife --target=es2022 --outfile=main.js
```

`tsc` は型を検査するだけ（`noEmit`）で、`main.js` を書き出すのは esbuild です。

Modelith に追加する前に、フォルダを検査します。SDK に、そのためのコマンドが入っています。

```sh
npx modelith-plugin validate
```

マニフェストの誤り、`main.js` が無いこと、`import` が残っていてまとめ直しが要る `main.js` などを知らせます。

## 5. Modelith に追加する

1. Chrome か Edge で <https://modelith.github.io/modelith/> を開き、`part def` がいくつかあるモデルのフォルダを開きます。
2. 「プラグイン → プラグインの一覧… → 追加…」を選び、`port-check` フォルダを選びます。
3. 追加先を選びます。「このプロジェクト」は `.modelith/plugins/com.example.port-check/` にコピーして `modelith.json` に登録するので、
   プロジェクトを開く人全員が使えます。「このブラウザ」は自分だけで、どのプロジェクトでも使えます。
4. Modelith が権限を示して、プラグインを動かしてよいかを尋ねます。許可します。

これで、次のことができます。

- ポートが 8 個より多い部品に、問題一覧で警告が出る
- 「プラグイン → プラグインのコマンド → モデルの概要を表示」で、数がトーストに出る
- 「プラグイン → プラグインでエクスポート → 要素の一覧（CSV）」で、`elements.csv` がダウンロードされる

プロジェクトごとに上限を変えるときは、`modelith.json` に設定を書きます。`activate` で届きます。

```json
{
  "plugins": {
    "com.example.port-check": { "version": "0.1.0", "settings": { "maxPorts": 4 } }
  }
}
```

## 6. 直して読み込み直す

`src/main.ts` を直したら、まとめ直してフォルダを追加し直します。同じ ID のプラグインを追加すると、置き換わります。
プロジェクトのプラグインなら、プロジェクトの中のコピーに直接書き出し、エクスプローラーの「一覧を更新」（↻）を押す方が早く回せます。

```sh
npx esbuild src/main.ts --bundle --format=iife --target=es2022 --outfile=../my-project/.modelith/plugins/com.example.port-check/main.js
```

宣言する権限が増えたとき、または `network` を宣言していてファイルが変わったときのほかは、許可を尋ね直しません。

プラグインの `console.log` の出力は、ブラウザの開発者ツールに出ます。

## 次に読むもの

- `proposeEdits` でテキストの変更を出す（`edits:propose` が必要）、`storageGet`・`storageSet` でデータを保存する（`storage` が必要）、
  `contributes.importers` でファイルを取り込む。
- プラグインにできること・できないこと、大きさの上限、Modelith が各メッセージをどう扱うかは、
  Modelith の [TypeScript のプラグインのリファレンス](https://github.com/modelith/modelith/blob/dev/docs/reference/script-plugins.md) にあります。
- メッセージの型と JSON Schema は、このリポジトリの [`schema/`](../../schema) と [`examples/`](../../examples) にあります。
