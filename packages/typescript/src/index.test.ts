import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import Ajv2020 from "ajv/dist/2020.js";
import { describe, expect, it } from "vitest";
import { isValidPluginId } from "./index.js";

const root = join(import.meta.dirname, "../../..");
const readJson = (...p: string[]) => JSON.parse(readFileSync(join(root, ...p), "utf8"));

describe("isValidPluginId", () => {
  // Rust 側（crates/modelith-plugin-protocol/src/id.rs）のテストと同じケース
  it.each(["com.example.fmea-check", "jp.modelith.iso15288-rules", "a.b"])("accepts %s", (id) => {
    expect(isValidPluginId(id)).toBe(true);
  });
  it.each(["", "single", "Com.example", "com..example", "com.example.", "1com.x", "com.ex_ample"])(
    "rejects %s",
    (id) => expect(isValidPluginId(id)).toBe(false),
  );
});

describe("JSON Schema accepts the shared examples", () => {
  const ajv = new Ajv2020.default({ strict: false });
  const manifest = ajv.compile(readJson("schema/plugin-manifest.schema.json"));
  const entry = ajv.compile(readJson("schema/registry-entry.schema.json"));

  const examples = readdirSync(join(root, "examples"));
  it.each(examples.filter((f) => f.startsWith("manifest.")))("%s", (f: string) => {
    expect(manifest(readJson("examples", f)), JSON.stringify(manifest.errors)).toBe(true);
  });
  it("registry-entry.json", () => {
    expect(entry(readJson("examples", "registry-entry.json")), JSON.stringify(entry.errors)).toBe(true);
  });
  it("rejects an invalid plugin id", () => {
    expect(manifest({ ...readJson("examples", "manifest.rules.json"), id: "Bad" })).toBe(false);
  });
});
