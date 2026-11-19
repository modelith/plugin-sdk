# Plugin author docs

[日本語](README.ja.md)

## Tutorials

Start here if you have not written a Modelith plugin before. Each tutorial goes from an empty folder to a plugin running in Modelith.

- [Your first rules plugin](tutorials/rules-plugin.md): check a model with YAML rules, in Modelith and from the command line.
- [Your first browser plugin](tutorials/browser-plugin.md): add a check, a command and an exporter in TypeScript.

## Reference

The exact behavior is specified in Modelith's documentation, and the types and schemas in this repository:

- [Check rules (rules plugins) and `modelith check`](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md)
- [TypeScript plugins (browser plugins)](https://github.com/modelith/modelith/blob/dev/docs/reference/script-plugins.md)
- [JSON Schemas](../schema) and [examples](../examples)
- [`@modelith/plugin-sdk`](../packages/typescript) (TypeScript) and [`modelith-plugin-protocol`](../crates/modelith-plugin-protocol) (Rust)
