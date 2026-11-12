# Changelog

All notable changes to the crate `modelith-plugin-protocol` and the npm package `@modelith/plugin-sdk` are recorded here.
Both are released together with the same version. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Modelith's versioning rules](https://github.com/modelith/modelith/blob/dev/docs/reference/versioning.md).
Before 1.0.0 (`apiVersion` `"0"`), any release may change the contract.

## [Unreleased]

## [0.1.0] - Unreleased

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
  `plugin-message` and `registry-entry`.
- `@modelith/plugin-sdk`: the generated TypeScript types, `connectToHost` for talking to Modelith from a Web Worker,
  `isValidPluginId`, and the `modelith-plugin validate` command that checks a plugin folder before you add it to Modelith.
- The Rust crate depends only on `serde` and `serde_json` by default. The `codegen` feature adds `schemars` and `ts-rs`
  for generating the schemas and TypeScript types.
- Tutorials for a first rules plugin and a first browser plugin, in English and Japanese.

[Unreleased]: https://github.com/modelith/plugin-sdk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/modelith/plugin-sdk/releases/tag/v0.1.0
