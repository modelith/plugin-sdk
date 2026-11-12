# Your first rules plugin

[日本語](rules-plugin.ja.md)

In this tutorial you write a plugin that checks a model against two rules, see the results in Modelith,
and run the same check from the command line. A rules plugin is a manifest and YAML files: no code, no build.

You need Modelith in Chrome or Edge (<https://modelith.github.io/modelith/>), which can open a folder on your computer.
The command-line part also needs [Rust](https://rustup.rs/).
Modelith's screens are in Japanese, so this tutorial quotes their labels with a translation.

## 1. Make a project with a model

A Modelith project is a folder with a `modelith.json`. Create this folder:

```
kettle/
  modelith.json
  model.sysml
```

`kettle/modelith.json` names the project and lists the plugins it uses:

```json
{
  "name": "Kettle",
  "plugins": {
    "com.example.kettle-checks": { "version": "0.1.0" }
  }
}
```

`kettle/model.sysml` has two requirements, and only one of them is satisfied:

```sysml
package Kettle {
    requirement def BoilWater;
    requirement def KeepWarm;
    part def Heater;
    satisfy requirement BoilWater by Heater;
}
```

## 2. Write the plugin

A plugin added to a project lives in `.modelith/plugins/<plugin id>/`. Create the folder and two files in it:

```
kettle/
  .modelith/
    plugins/
      com.example.kettle-checks/
        modelith-plugin.json
        rules.yaml
```

`modelith-plugin.json` is the manifest. The id is in reverse-domain form and must match the folder name.
`contributes.rules` lists the rule files, relative to the plugin folder.

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

`rules.yaml` has the rules. Each rule picks elements with `match`, and reports every picked element that does not meet `require`:

```yaml
rules:
  - id: req-has-satisfier
    severity: warning
    message: "Nothing satisfies requirement {name}"
    match:
      kind: requirement def
    require:
      incoming: satisfy
  - id: part-has-ports
    severity: information
    message: "{kind} {name} has no ports"
    match:
      kind: [part def, part]
    require:
      ports: { min: 1 }
```

- `req-has-satisfier`: every `requirement def` receives at least one `satisfy` relation.
- `part-has-ports`: every `part def` or `part` has at least one port.

## 3. See the results in Modelith

1. Open <https://modelith.github.io/modelith/> in Chrome or Edge and open the `kettle` folder.
2. Open `model.sysml`. The problems list (問題一覧) under the text shows:
   - a warning on `KeepWarm`: nothing satisfies it;
   - information on `Heater`: it has no ports.
3. Add `satisfy requirement KeepWarm by Heater;` to the model. The warning goes away as you type.

プラグイン → プラグインの一覧… (Plugins → Plugin list…) shows the plugin, its rule count and any errors in the manifest or the rules.
A mistake such as an unknown key or a broken regular expression is reported with its file, line and column, never silently ignored.

## 4. Check from the command line

The `modelith` command runs the same rules, so you can run them in CI too.
Until it ships in a release, install it from the source:

```sh
cargo install --git https://github.com/modelith/modelith --branch dev modelith-cli
modelith check kettle
```

With the model from step 1, it prints one line per result (path, line and column, severity, message, then `[plugin id/rule id]`)
and a summary. The severity labels and the summary are in Japanese for now:

```
model.sysml:3:5: 警告: Nothing satisfies requirement KeepWarm [com.example.kettle-checks/req-has-satisfier]
model.sysml:4:5: 情報: part def Heater has no ports [com.example.kettle-checks/part-has-ports]
ファイル 1 件をチェックしました：エラー 0 件・警告 1 件・情報 1 件・ヒント 0 件
```

The exit code is 1 when there is at least one `error`, and 0 otherwise. Change `severity` to `error` to make CI fail on a rule.

## What's next

- Share the plugin by committing `.modelith/` with the project. Everyone who opens the project, and CI, uses the same rules.
- To use the plugin in every project in your browser instead, keep the plugin folder elsewhere and add it with
  プラグイン → プラグインの一覧… → 追加… (Plugins → Plugin list… → Add…), choosing このブラウザ (This browser).
  The command line does not see plugins added this way.
- Everything a rule can say — the element kinds, relation kinds, `phase`, `name` patterns and `not` — is in
  Modelith's [check rule reference](https://github.com/modelith/modelith/blob/dev/docs/reference/check-rules.md).
- When a rule is not enough, write [a browser plugin](browser-plugin.md).
