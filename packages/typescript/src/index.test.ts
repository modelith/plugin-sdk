import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import Ajv2020 from "ajv/dist/2020.js";
import { describe, expect, it } from "vitest";
import { isValidPluginId } from "./index.js";

const root = join(import.meta.dirname, "../../..");
const readJson = (...p: string[]) => JSON.parse(readFileSync(join(root, ...p), "utf8"));

describe("isValidPluginId", () => {
  // Rust 側（crates/modelith-plugin-protocol/src/id.rs）のテストと同じ同値クラス・境界値
  it.each(["com.example.fmea-check", "jp.modelith.iso15288-rules", "a.b"])("accepts %s", (id: string) => {
    // Arrange（正常系: 逆ドメイン形式。a.b は最短の境界値）
    const candidate = id;
    // Act
    const valid = isValidPluginId(candidate);
    // Assert
    expect(valid).toBe(true);
  });

  it.each(["", "single", "Com.example", "com..example", "com.example.", "1com.x", "com.ex_ample"])(
    "rejects %j",
    (id: string) => {
      // Arrange（異常系: 区切り不足・大文字・空の区切り・数字始まり・使えない文字）
      const candidate = id;
      // Act
      const valid = isValidPluginId(candidate);
      // Assert
      expect(valid).toBe(false);
    },
  );
});

describe("JSON Schema accepts the shared examples", () => {
  const ajv = new Ajv2020.default({ strict: false });
  const manifest = ajv.compile(readJson("schema/plugin-manifest.schema.json"));
  const entry = ajv.compile(readJson("schema/registry-entry.schema.json"));
  const manifests = readdirSync(join(root, "examples")).filter((f) => f.startsWith("manifest."));

  it.each(manifests)("%s", (file: string) => {
    // Arrange
    const json = readJson("examples", file);
    // Act
    const valid = manifest(json);
    // Assert
    expect(valid, JSON.stringify(manifest.errors)).toBe(true);
  });

  it("registry-entry.json", () => {
    // Arrange
    const json = readJson("examples", "registry-entry.json");
    // Act
    const valid = entry(json);
    // Assert
    expect(valid, JSON.stringify(entry.errors)).toBe(true);
  });

  it("rejects an invalid plugin id", () => {
    // Arrange（異常系: スキーマのパターンに反する id）
    const json = { ...readJson("examples", "manifest.rules.json"), id: "Bad" };
    // Act
    const valid = manifest(json);
    // Assert
    expect(valid).toBe(false);
  });
});
