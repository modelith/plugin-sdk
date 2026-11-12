<p align="center">
  <img src="https://raw.githubusercontent.com/modelith/.github/main/profile/logo.svg" width="88" height="88" alt="Modelith">
</p>

<h1 align="center">Modelith Plugin SDK</h1>

<p align="center">
  <b>Types, JSON Schemas and helpers for building <a href="https://github.com/modelith/modelith">Modelith</a> plugins</b><br>
  <sub>Modelith のプラグインを作るための型・JSON Schema・ヘルパー</sub>
</p>

<p align="center">
  <a href="README.ja.md">日本語</a> &nbsp;|&nbsp;
  <a href="#quick-start">Quick start</a> &nbsp;|&nbsp;
  <a href="packages/typescript">TypeScript</a> &nbsp;|&nbsp;
  <a href="crates/modelith-plugin-protocol">Rust</a> &nbsp;|&nbsp;
  <a href="schema">JSON Schema</a> &nbsp;|&nbsp;
  <a href="CONTRIBUTING.md">Contributing</a> &nbsp;|&nbsp;
  <a href="https://github.com/sponsors/modelith">Sponsor</a>
</p>

<p align="center">
  <a href="https://github.com/modelith/plugin-sdk/actions/workflows/ci.yml?query=branch%3Adev"><img src="https://github.com/modelith/plugin-sdk/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI"></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT-blue" alt="license"></a>
</p>

Modelith is an open-source SysML v2 modeling editor with synchronized text and diagram editing.
This repository defines the **contract between Modelith and its plugins**: the plugin manifest,
the rule file format, the messages a plugin exchanges with the host, and the plugin registry entry.
The editor, plugin authors and the registry's CI all build on it.

> [!NOTE]
> The SDK is pre-1.0 (`apiVersion` `"0"`): the contract may still change between minor releases.
> The packages are not on npm or crates.io yet — see [Install](#install).

## What a plugin can be

| `runtime` | Written as | Runs in | Good for |
| --- | --- | --- | --- |
| `rules` | YAML rule files | Browser, CLI, CI (Modelith interprets them) | Model checks: "every requirement is satisfied by something" |
| `browser` | A single bundled script | A Web Worker inside a sandboxed iframe | Importers, exporters, views, commands, custom checks |
| `wasm-component` | A WebAssembly component | Browser, CLI, server | Headless plugins (planned) |

A plugin declares what it adds (`contributes`) and what it needs (`permissions`) in its manifest,
`modelith-plugin.json`. Modelith refuses anything a plugin did not declare.

## Quick start

### A rules plugin

`modelith-plugin.json`:

```json
{
  "id": "com.example.req-checks",
  "name": "Requirement checks",
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
    message: "Nothing satisfies requirement {name}"
    match:
      kind: requirement def
    require:
      incoming: satisfy
```

The full rule vocabulary is in [`schema/rule-set.schema.json`](schema/rule-set.schema.json) and
Modelith's [check rule reference](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md).

### A browser plugin

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
      // Inspect message.text, or message.model when the manifest asks for "model:read".
      host.send({ type: "diagnostics", version: message.version, items });
      break;
    }
  }
});
```

Bundle it into one classic script — Chromium cannot run a module script inside the sandboxed iframe — and point `main` at it:

```sh
npx esbuild src/index.ts --bundle --format=iife --outfile=dist/index.js
```

```json
{ "runtime": "browser", "main": "dist/index.js", "permissions": ["model:read"] }
```

See [`examples/`](examples) for complete manifests, rule files and a full message exchange.

## Install

Until the first release, use the Git repository:

```toml
# Rust (Cargo.toml)
[dependencies]
modelith-plugin-protocol = { git = "https://github.com/modelith/plugin-sdk", branch = "dev" }
```

For TypeScript, build the package from source (`npm ci && npm run build` in `packages/typescript`).
After the first release:

```sh
npm install @modelith/plugin-sdk
cargo add modelith-plugin-protocol
```

## What's in this repository

| Path | Contents |
| --- | --- |
| [`crates/modelith-plugin-protocol`](crates/modelith-plugin-protocol) | The single source of truth: the contract as Rust types |
| [`schema/`](schema) | JSON Schemas generated from the Rust types |
| [`packages/typescript`](packages/typescript) | `@modelith/plugin-sdk`: generated TypeScript types and helpers |
| [`examples/`](examples) | Sample manifests, rule file, registry entry and messages, checked against both Rust and the schemas |

| Schema | Describes |
| --- | --- |
| [`plugin-manifest`](schema/plugin-manifest.schema.json) | `modelith-plugin.json` |
| [`rule-set`](schema/rule-set.schema.json) | Rule files of a `rules` plugin |
| [`host-message`](schema/host-message.schema.json) / [`plugin-message`](schema/plugin-message.schema.json) | Messages between Modelith and a `browser` plugin |
| [`registry-entry`](schema/registry-entry.schema.json) | An entry in the plugin registry |

## Versioning

`apiVersion` in a manifest is the SDK's major version. While it is `"0"`, nothing is guaranteed to stay
compatible. From 1.0.0 on, removing or renaming anything, or adding a required field, needs a new major version.
The full rules are in Modelith's [versioning guide](https://github.com/modelith/modelith/blob/dev/docs/reference/versioning.md),
and the design behind the plugin host is in [ADR-0018](https://github.com/modelith/modelith/blob/dev/docs/adr/0018-plugin-host.md).

## Contributing

Bug reports and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) explains how to build,
test and change the contract.

## License

MIT. Plugins that use only these interfaces may choose any license
(see the [Modelith Plugin Exception](https://github.com/modelith/modelith/blob/main/PLUGIN-EXCEPTION.md)).
