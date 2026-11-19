// 文書（README と手引き）に載せたコード例が、今の型・スキーマ・SDK で動くことを確かめる
// （結合テスト: 文書 → コード例を取り出す → tsc・JSON Schema・実行）。SDK を変えて例が古くなったら、ここで落ちる。

import { execFileSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import Ajv2020 from "ajv/dist/2020.js";
import { afterAll, describe, expect, it } from "vitest";
import { parse as parseYaml } from "yaml";
import type { HostMessage, PluginMessage } from "./index.js";

const root = join(import.meta.dirname, "../../..");
const src = join(import.meta.dirname, "index.ts");
const readJson = (...p: string[]) => JSON.parse(readFileSync(join(root, ...p), "utf8"));

/** コード例を載せている文書（リポジトリのルートからのパス）。 */
const documents = [
  "README.md",
  "README.ja.md",
  "packages/typescript/README.md",
  ...readdirSync(join(root, "docs"), { recursive: true, encoding: "utf8" })
    .filter((p) => p.endsWith(".md"))
    .map((p) => join("docs", p)),
];

/** 文書の中の、指定した言語のコードブロックを並べる。 */
function codeBlocks(language: string): { doc: string; index: number; code: string }[] {
  return documents.flatMap((doc) =>
    [...readFileSync(join(root, doc), "utf8").matchAll(new RegExp("^```" + language + "\\n([\\s\\S]*?)^```$", "gm"))].map(
      (m, index) => ({ doc, index, code: m[1] ?? "" }),
    ),
  );
}

const work = mkdtempSync(join(tmpdir(), "plugin-sdk-docs-"));
afterAll(() => rmSync(work, { recursive: true, force: true }));

describe("TypeScript examples in the docs", () => {
  it("type-check as Web Worker code against the current SDK", () => {
    // Arrange（例ごとに 1 ファイル。パッケージ名は src に向け、ビルド前の SDK で検査する）
    const files = codeBlocks("ts").map(({ doc, index, code }) => {
      const file = join(work, `${doc.replaceAll(/[/.]/g, "_")}_${index}.ts`);
      writeFileSync(file, code);
      return file;
    });
    const tsconfig = join(work, "tsconfig.json");
    writeFileSync(
      tsconfig,
      JSON.stringify({
        compilerOptions: {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          lib: ["ES2022", "WebWorker"],
          types: [],
          strict: true,
          noEmit: true,
          skipLibCheck: true,
          resolveJsonModule: true,
          paths: {
            "@modelith/plugin-sdk": [src],
            "@modelith/plugin-sdk/schema/*": [join(root, "schema/*")],
          },
        },
        files,
      }),
    );
    // Act
    let errors = "";
    try {
      execFileSync(join(import.meta.dirname, "../node_modules/.bin/tsc"), ["-p", tsconfig], { encoding: "utf8" });
    } catch (e) {
      errors = String((e as { stdout?: unknown }).stdout ?? e);
    }
    // Assert（失敗したら tsc の出力をそのまま見せる。ファイル名から文書と何番目の例かが分かる）
    expect(files.length).toBeGreaterThan(0);
    expect(errors).toBe("");
  });
});

describe("JSON examples in the docs", () => {
  const ajv = new Ajv2020.default({ strict: false, logger: false });
  const manifest = ajv.compile(readJson("schema/plugin-manifest.schema.json"));
  const blocks = codeBlocks("json");

  it.each(blocks.map((b) => [`${b.doc}[${b.index}]`, b.code] as const))("%s is valid JSON", (_: string, code: string) => {
    // Arrange
    const text = code;
    // Act
    const parse = () => JSON.parse(text);
    // Assert
    expect(parse).not.toThrow();
  });

  // apiVersion を持つものは完全なマニフェスト（一部だけを見せる例は対象外）
  const manifests = blocks.filter((b) => JSON.parse(b.code).apiVersion !== undefined);

  it.each(manifests.map((b) => [`${b.doc}[${b.index}]`, b.code] as const))("%s follows the manifest schema", (_: string, code: string) => {
    // Arrange
    const json: unknown = JSON.parse(code);
    // Act
    const valid = manifest(json);
    // Assert
    expect(valid, JSON.stringify(manifest.errors)).toBe(true);
  });
});

describe("YAML examples in the docs", () => {
  const ajv = new Ajv2020.default({ strict: false, logger: false });
  const ruleSet = ajv.compile(readJson("schema/rule-set.schema.json"));

  it.each(codeBlocks("yaml").map((b) => [`${b.doc}[${b.index}]`, b.code] as const))("%s follows the rule set schema", (_: string, code: string) => {
    // Arrange
    const rules: unknown = parseYaml(code);
    // Act
    const valid = ruleSet(rules);
    // Assert
    expect(valid, JSON.stringify(ruleSet.errors)).toBe(true);
  });
});

describe.each(["docs/tutorials/browser-plugin.md", "docs/tutorials/browser-plugin.ja.md"])("the plugin in %s", (doc) => {
  const code = codeBlocks("ts").find((b) => b.doc === doc)?.code ?? "";
  let loaded = 0;

  /** 手引きのプラグインを Worker の代わりに動かし、ホストの役で `messages` を送って、返ってきたものを集める。 */
  async function talk(messages: HostMessage[]): Promise<PluginMessage[]> {
    const file = join(work, `${doc.replaceAll(/[/.]/g, "_")}_run${loaded++}.ts`);
    writeFileSync(file, code.replace('"@modelith/plugin-sdk"', JSON.stringify(src)));
    const listeners: ((event: { data: unknown }) => void)[] = [];
    const sent: PluginMessage[] = [];
    Object.assign(globalThis, {
      self: {
        postMessage: (m: PluginMessage) => sent.push(m),
        addEventListener: (_: string, l: (event: { data: unknown }) => void) => listeners.push(l),
        removeEventListener: () => {},
      },
    });
    await import(pathToFileURL(file).href);
    for (const data of messages) listeners.forEach((l) => l({ data }));
    return sent;
  }

  const range = { start: { line: 0, character: 0 }, end: { line: 0, character: 4 } };
  const part = (name: string, ports: number) => ({
    key: name,
    name,
    kind: "part def",
    ports: Array.from({ length: ports }, (_, i) => ({ name: `p${i}` })),
    range,
  });
  const activate = (settings?: unknown): HostMessage => ({
    type: "activate",
    apiVersion: "0",
    pluginId: "com.example.port-check",
    ...(settings === undefined ? {} : { settings }),
  });
  const didChange = (...elements: ReturnType<typeof part>[]): HostMessage => ({
    type: "didChange",
    version: 3,
    text: "",
    model: { elements, relations: [] },
  });

  it("answers activate with ready", async () => {
    // Arrange
    const messages = [activate()];
    // Act
    const sent = await talk(messages);
    // Assert
    expect(sent).toEqual([{ type: "ready" }]);
  });

  it("warns only on parts with more ports than the default limit of 8", async () => {
    // Arrange（境界値: 上限ちょうどの 8 は出さず、9 で出す）
    const messages = [activate(), didChange(part("AtLimit", 8), part("OverLimit", 9))];
    // Act
    const sent = await talk(messages);
    // Assert
    const diagnostics = sent.find((m) => m.type === "diagnostics");
    expect(diagnostics).toMatchObject({ version: 3, items: [{ severity: "warning", code: "too-many-ports", range }] });
    expect(diagnostics?.type === "diagnostics" && diagnostics.items.length).toBe(1);
  });

  it("takes the limit from the project settings", async () => {
    // Arrange（設定 maxPorts: 2。境界値: 2 は出さず、3 で出す）
    const messages = [activate({ maxPorts: 2 }), didChange(part("AtLimit", 2), part("OverLimit", 3))];
    // Act
    const sent = await talk(messages);
    // Assert
    const diagnostics = sent.find((m) => m.type === "diagnostics");
    expect(diagnostics?.type === "diagnostics" && diagnostics.items.length).toBe(1);
  });

  it("runs the summary command and reports it done", async () => {
    // Arrange
    const messages = [activate(), didChange(part("A", 0)), { type: "runCommand", requestId: 7, command: "summary" } as const];
    // Act
    const sent = await talk(messages);
    // Assert
    expect(sent.slice(-2)).toMatchObject([{ type: "notify", level: "info" }, { type: "commandDone", requestId: 7 }]);
  });

  it("exports the elements as UTF-8 CSV", async () => {
    // Arrange（日本語の名前も壊れずに base64 になること）
    const messages = [activate(), didChange(part("電球", 0)), { type: "export", requestId: 8, exporter: "elements-csv" } as const];
    // Act
    const sent = await talk(messages);
    // Assert
    const exported = sent.at(-1);
    expect(exported).toMatchObject({ type: "exported", requestId: 8, fileName: "elements.csv", mediaType: "text/csv" });
    const csv = exported?.type === "exported" ? Buffer.from(exported.base64, "base64").toString("utf8") : "";
    expect(csv).toBe('name,kind,phase\n"電球","part def",""\n');
  });

  it("fails an export before the model is analyzed", async () => {
    // Arrange（異常系: didChange の前にエクスポートを頼まれる）
    const messages = [activate(), { type: "export", requestId: 9, exporter: "elements-csv" } as const];
    // Act
    const sent = await talk(messages);
    // Assert
    expect(sent.at(-1)).toMatchObject({ type: "failed", requestId: 9 });
  });
});
