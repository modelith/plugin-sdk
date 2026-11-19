#!/usr/bin/env node
// `modelith-plugin` コマンドの入口。実際のファイルと、パッケージに入れた JSON Schema を使って run を呼ぶ。

import { readFileSync } from "node:fs";
import { run } from "./cli.js";

const readFile = (path: string | URL): string | undefined => {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return undefined;
  }
};
const schema = (name: string): object => JSON.parse(readFileSync(new URL(`../schema/${name}.schema.json`, import.meta.url), "utf8"));

process.exitCode = run(process.argv.slice(2), {
  readFile,
  out: (line) => console.log(line),
  schemas: { manifest: schema("plugin-manifest"), ruleSet: schema("rule-set") },
});
