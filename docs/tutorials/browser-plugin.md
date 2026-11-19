# Your first browser plugin

[日本語](browser-plugin.ja.md)

In this tutorial you write a TypeScript plugin that adds three things to Modelith:

- a check: a warning on every part with more ports than a limit, which each project can set;
- a command that shows how many elements and relations the model has;
- an exporter that saves the model's elements as a CSV file.

Along the way you see how a plugin talks to Modelith, bundles its code, and asks for permissions.
If all you need is a check, a [rules plugin](rules-plugin.md) is simpler.

You need [Node.js](https://nodejs.org/), Git, and Modelith in Chrome or Edge (<https://modelith.github.io/modelith/>).
Modelith's screens are in Japanese, so this tutorial quotes their labels with a translation.

## 1. Set up the project

Until the first release of the SDK, build it from this repository:

```sh
git clone https://github.com/modelith/plugin-sdk
npm ci --prefix plugin-sdk/packages/typescript
npm run build --prefix plugin-sdk/packages/typescript
```

Then create the plugin next to it:

```sh
mkdir port-check && cd port-check
npm init -y
npm install --save-dev esbuild typescript ../plugin-sdk/packages/typescript
```

After the first release, install the SDK from its GitHub Release instead of the clone and the build
(it is not on the npm registry, but it installs as `@modelith/plugin-sdk`):

```sh
npm install --save-dev esbuild typescript https://github.com/modelith/plugin-sdk/releases/download/v0.1.0/modelith-plugin-sdk-0.1.0.tgz
```

You end up with this layout. `main.js` is the bundled plugin that Modelith runs.

```
port-check/
  modelith-plugin.json
  package.json
  tsconfig.json
  src/main.ts
  main.js          ← built in step 4
```

## 2. Write the manifest

`modelith-plugin.json` tells Modelith what the plugin adds and what it needs.

```json
{
  "id": "com.example.port-check",
  "name": "Port check",
  "version": "0.1.0",
  "apiVersion": "0",
  "runtime": "browser",
  "author": "Your Name",
  "license": "MIT",
  "main": "main.js",
  "permissions": ["model:read"],
  "contributes": {
    "commands": [{ "id": "summary", "title": "Show a model summary" }],
    "exporters": [{ "id": "elements-csv", "label": "Elements (CSV)", "extension": ".csv" }]
  }
}
```

- `permissions`: `model:read` lets the plugin receive the analyzed model and report diagnostics.
  Modelith shows the permissions when it asks the user whether to run the plugin, and refuses anything not declared.
- `contributes.commands` and `contributes.exporters` appear in Modelith's プラグイン (Plugins) menu.

## 3. Write the plugin

`tsconfig.json` makes TypeScript check the code as a Web Worker:

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

`src/main.ts`:

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
        host.send({ type: "failed", requestId: message.requestId, message: "The model has not been analyzed yet." });
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
      message: `${element.name} has ${element.ports.length} ports (at most ${maxPorts})`,
      code: "too-many-ports",
    }));
}

function summary(model: ModelSnapshot | undefined): string {
  if (!model) return "The model has not been analyzed yet.";
  return `${model.elements.length} elements, ${model.relations.length} relations`;
}

function toCsv(model: ModelSnapshot): string {
  const quote = (value: string) => `"${value.replaceAll('"', '""')}"`;
  const rows = model.elements.map((e) => [e.name, e.kind, e.phase ?? ""].map(quote).join(","));
  return ["name,kind,phase", ...rows].join("\n") + "\n";
}

// btoa() takes Latin-1 only, so encode the text as UTF-8 bytes first.
function toBase64(text: string): string {
  let binary = "";
  for (const byte of new TextEncoder().encode(text)) binary += String.fromCharCode(byte);
  return btoa(binary);
}
```

How the conversation goes:

1. Modelith sends `activate` with the project's settings. The plugin answers `ready`.
2. Modelith sends `didChange` with the text and, because of `model:read`, the analyzed model.
   It sends it again each time the text is analyzed. The plugin answers with `diagnostics` for that `version`;
   diagnostics for an older version are dropped.
3. When the user picks the command or the exporter, Modelith sends `runCommand` or `export` with a `requestId`.
   The plugin answers with the same `requestId`: `commandDone`, `exported`, or `failed`.

Every message type is in `HostMessage` and `PluginMessage`, so the editor completes and checks them.

## 4. Bundle it

The plugin runs as a classic script in a Worker inside a sandboxed iframe, where `import` cannot load anything.
Bundle the code and the SDK into one file:

```sh
npx tsc
npx esbuild src/main.ts --bundle --format=iife --target=es2022 --outfile=main.js
```

`tsc` only checks types (`noEmit`); esbuild writes `main.js`.

Check the folder before adding it to Modelith. The SDK comes with a command for that:

```sh
npx modelith-plugin validate
```

It reports mistakes in the manifest, a missing `main.js`, or a `main.js` that still uses `import` and needs bundling.

## 5. Add it to Modelith

1. Open <https://modelith.github.io/modelith/> in Chrome or Edge and open a folder that has a model with a few `part def`s.
2. Choose プラグイン → プラグインの一覧… → 追加… (Plugins → Plugin list… → Add…) and pick the `port-check` folder.
3. Choose where to add it. このプロジェクト (This project) copies the plugin into `.modelith/plugins/com.example.port-check/` and registers it in `modelith.json`,
   so everyone who opens the project gets it. このブラウザ (This browser) keeps it for you only, in every project.
4. Modelith asks whether to run the plugin and shows its permissions. Allow it.

Now:

- parts with more than 8 ports get a warning in the problems list;
- プラグイン → プラグインのコマンド → Show a model summary (Plugins → Plugin commands) shows a toast with the counts;
- プラグイン → プラグインでエクスポート → Elements (CSV) (Plugins → Export with a plugin) downloads `elements.csv`.

To change the limit for a project, add settings to its `modelith.json`. They arrive in `activate`:

```json
{
  "plugins": {
    "com.example.port-check": { "version": "0.1.0", "settings": { "maxPorts": 4 } }
  }
}
```

## 6. Change and reload

After you edit `src/main.ts`, bundle again and add the folder again. Adding a plugin with the same id replaces it.
For a faster loop with a project plugin, write the bundle straight into the project's copy, then press 一覧を更新 (↻, Refresh) in the explorer:

```sh
npx esbuild src/main.ts --bundle --format=iife --target=es2022 --outfile=../my-project/.modelith/plugins/com.example.port-check/main.js
```

Modelith does not ask for permission again unless the plugin declares a new permission, or it declares `network` and its file changes.

Messages from `console.log` in the plugin appear in the browser's developer tools.

## What's next

- Propose text changes with `proposeEdits` (needs `edits:propose`), keep data with `storageGet` / `storageSet` (needs `storage`),
  or import files with `contributes.importers`.
- What a plugin can and cannot do, size limits, and exactly how Modelith handles each message are in Modelith's
  [TypeScript plugin reference](https://github.com/modelith/modelith/blob/dev/docs/reference/script-plugins.md).
- The message types and their JSON Schemas are in this repository: [`schema/`](../../schema) and [`examples/`](../../examples).
