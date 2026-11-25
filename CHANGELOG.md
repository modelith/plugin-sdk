# Changelog

All notable changes to the crate `modelith-plugin-protocol` and the TypeScript package `modelith-plugin-sdk` are recorded here.
Both are released together with the same version: the crate on crates.io, the TypeScript package on npm and attached to the GitHub Release.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Modelith's versioning rules](https://github.com/modelith/modelith/blob/dev/docs/reference/versioning.md).
Before 1.0.0 (`apiVersion` `"0"`), any release may change the contract.

## [Unreleased]

### Added

- The combined registry index that the app fetches: `RegistryIndex` (`plugins`, a list of `RegistryEntry`),
  with the JSON Schema `registry-index`. Unknown top-level fields are ignored.

## [0.2.0] - 2026-11-24

### Changed

- **Breaking:** the TypeScript package is renamed from `@modelith/plugin-sdk` to `modelith-plugin-sdk` (Modelith's ADR-0028),
  because the npm scope `@modelith` is not available. Change imports from `@modelith/plugin-sdk` to `modelith-plugin-sdk`
  and from `@modelith/plugin-sdk/schema/*` to `modelith-plugin-sdk/schema/*`. The crate, the types, the schemas and
  `apiVersion` do not change.
- The TypeScript package is published to the npm registry (`npm install modelith-plugin-sdk`).
  It is still attached to each GitHub Release.

### Added

- Outputs of reading and checking a model, shared by AI clients (MCP), the app and the server (Modelith's ADR-0024):
  `ProjectOutline` (files, elements, diagrams), `ElementSummary`, `ElementDetail` (doc, type, phase, members, ports,
  relations), `ElementRelation`, `DiagramSummary` and `ProblemReport`, with the JSON Schemas `project-outline`,
  `element-detail` and `problem-report`. Unknown fields are ignored.
- Inputs and outputs of writing a model, in two steps (propose, then write; Modelith's ADR-0024): `EditRequest`
  (`addElement`, `addRelation`, `rename`, `delete`, `setDoc`, `replaceText`, pointing at elements by qualified name),
  `EditProposal` (file changes and the problems after applying them) and `FileChange` (`TextEdit`s on a `baseVersion`),
  with the JSON Schemas `edit-request` and `edit-proposal`. Unknown fields are ignored.

## [0.1.0] - 2026-11-19

The first release.

### Added

- The plugin manifest (`modelith-plugin.json`): `PluginManifest` with `id`, `name`, `version`, `apiVersion`, `runtime`
  (`rules`, `browser`, `wasm-component`), `description`, `author`, `license`, `engines.modelith`, `main`, `permissions`
  (`model:read`, `edits:propose`, `network`, `storage`) and `contributes` (rules, importers, exporters, views, palette, commands).
  Unknown fields are rejected.
- Rule files for `rules` plugins: `RuleSet`, with `match` (kind, phase, name) and `require`
  (incoming, outgoing, name, phase, ports, not). Unknown fields are rejected.
- Messages between Modelith and a `browser` plugin: `HostMessage`, `PluginMessage` and the `ModelSnapshot` sent with `didChange`.
  Unknown fields are ignored, so a host and a plugin of different versions keep working.
- `TextEdit` and `Diagnostic`, in the same shape as the Language Server Protocol.
- The plugin registry entry: `RegistryEntry`.
- JSON Schemas for all of the above, generated from the Rust types: `plugin-manifest`, `rule-set`, `host-message`,
  `plugin-message`, `registry-entry` and `usage-report`.
- The usage report sent by the Modelith app and server when the user allows it (Modelith's ADR-0026): `UsageReport`
  (`kind`: `startup`, `daily`, `error`, `serverDaily`), carrying `schemaVersion` (`USAGE_SCHEMA_VERSION`) and the sender
  (`client`, `appVersion`, `channel`). Values are fixed words, classes or counts up to `COUNT_MAX`, never free text, and plugins are counted without their names.
  `usage-report.schema.json` annotates each report with `x-modelith-when`, `x-modelith-level` and `x-modelith-max-bytes`,
  and the sender fields with `x-modelith-dimension`. Unknown fields are ignored.
- `@modelith/plugin-sdk`: the generated TypeScript types, `connectToHost` for talking to Modelith from a Web Worker,
  `isValidPluginId`, and the `modelith-plugin validate` command that checks a plugin folder before you add it to Modelith.
- The Rust crate depends only on `serde` and `serde_json` by default. The `codegen` feature adds `schemars` and `ts-rs`
  for generating the schemas and TypeScript types.
- Tutorials for a first rules plugin and a first browser plugin, in English and Japanese.

[Unreleased]: https://github.com/modelith/plugin-sdk/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/modelith/plugin-sdk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/modelith/plugin-sdk/releases/tag/v0.1.0
