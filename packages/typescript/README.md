# @modelith/plugin-sdk

TypeScript types, JSON Schemas and helpers for building [Modelith](https://github.com/modelith/modelith) plugins.

The types are generated from the Rust crate [`modelith-plugin-protocol`](../../crates/modelith-plugin-protocol),
the single source of truth for the contract between Modelith and its plugins.
See the [repository README](../../README.md) for an overview of plugin types.

> [!NOTE]
> Pre-1.0 (`apiVersion` `"0"`): the contract may still change between minor releases.

## Install

```sh
npm install @modelith/plugin-sdk
```

Until the first release, build it from the repository: `npm ci && npm run build` in `packages/typescript`.

## Talk to Modelith from a `browser` plugin

A `browser` plugin runs in a Web Worker. `connectToHost` gives it typed messages in both directions:

```ts
import { connectToHost } from "@modelith/plugin-sdk";

const host = connectToHost(self);

const stop = host.onMessage((message) => {
  if (message.type === "activate") host.send({ type: "ready" });
  if (message.type === "runCommand") host.send({ type: "commandDone", requestId: message.requestId });
});
```

- `onMessage(handler)` receives every `HostMessage` and returns a function that stops receiving.
  Data without a string `type` (messages from other libraries) is ignored; the contents are not validated.
- `send(message)` posts a `PluginMessage` to Modelith.

Bundle the plugin into a single classic script (for example `esbuild --bundle --format=iife`).
Chromium cannot run a module script inside the sandboxed iframe that hosts the worker.

## Types

Everything in the contract is exported as a type:

| Area | Types |
| --- | --- |
| Manifest | `PluginManifest`, `Runtime`, `Permission`, `Contributes`, `Engines`, … |
| Rule files | `RuleSet`, `Rule`, `ElementMatch`, `Requirement`, … |
| Messages | `HostMessage`, `PluginMessage`, `ModelSnapshot`, `ModelElement`, `ModelRelation`, … |
| Edits and diagnostics | `TextEdit`, `Diagnostic`, `Range`, `Position` |
| Registry | `RegistryEntry`, `RegistryVersion`, `Publisher`, `Pricing` |

Constants and helpers:

| Name | What it is |
| --- | --- |
| `API_VERSION` | The `apiVersion` a manifest must declare (`"0"`) |
| `PLUGIN_ID_PATTERN`, `RULE_ID_PATTERN` | Regular expressions for plugin and rule ids |
| `isValidPluginId(id)` | Whether `id` is a reverse-domain plugin id such as `com.example.my-plugin` |

## JSON Schemas

The package ships the JSON Schemas too, for editors and CI:

```ts
import manifestSchema from "@modelith/plugin-sdk/schema/plugin-manifest.schema.json" with { type: "json" };
```

Available: `plugin-manifest`, `rule-set`, `host-message`, `plugin-message`, `registry-entry`.

## License

MIT
