// @modelith/plugin-sdk — Modelith プラグイン向けの型とヘルパー。
// 型は crates/modelith-plugin-protocol から生成している（src/generated/ は編集しない）。

export * from "./generated/protocol.js";

import { PLUGIN_ID_PATTERN } from "./generated/protocol.js";

const pluginIdRe = new RegExp(PLUGIN_ID_PATTERN);

/** 逆ドメイン形式のプラグイン ID かどうか（例: `com.example.my-plugin`）。 */
export function isValidPluginId(id: string): boolean {
  return pluginIdRe.test(id);
}
