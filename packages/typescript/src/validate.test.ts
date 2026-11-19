import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { type Io, run } from "./cli.js";
import { formatProblem, isInsideFolder, looksLikeModule, type PluginFolder, type Problem, validatePlugin } from "./validate.js";

const root = join(import.meta.dirname, "../../..");
const readJson = (name: string): object => JSON.parse(readFileSync(join(root, "schema", `${name}.schema.json`), "utf8"));
const schemas = { manifest: readJson("plugin-manifest"), ruleSet: readJson("rule-set") };

const RULES = `rules:
  - id: req-has-satisfier
    severity: warning
    message: "Nothing satisfies {name}"
    match: { kind: requirement def }
    require: { incoming: satisfy }
`;
const rulesManifest = {
  id: "com.example.checks",
  name: "Checks",
  version: "0.1.0",
  apiVersion: "0",
  runtime: "rules",
  license: "MIT",
  contributes: { rules: ["rules.yaml"] },
};
const browserManifest = {
  id: "com.example.tool",
  name: "Tool",
  version: "0.1.0",
  apiVersion: "0",
  runtime: "browser",
  license: "MIT",
  main: "main.js",
  contributes: { commands: [{ id: "run", title: "Run" }] },
};
const IIFE = '"use strict";\n(() => {\n  self.postMessage({ type: "ready" });\n})();\n';

/** ファイル名 → 中身 のフォルダ。manifest にオブジェクトを渡せば JSON にする。 */
function folder(files: Record<string, string | object>, options: Partial<PluginFolder> = {}): PluginFolder {
  return {
    name: "checks",
    inProject: false,
    read: (path) => {
      const content = files[path];
      return typeof content === "object" ? JSON.stringify(content) : content;
    },
    ...options,
  };
}

const messages = (problems: Problem[]) => problems.map(formatProblem);

describe("validatePlugin", () => {
  // --- 正常系 ---

  it("accepts a rules plugin with a valid rule file", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": rulesManifest, "rules.yaml": RULES });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(problems).toEqual([]);
  });

  it("accepts a browser plugin whose main is a bundled classic script", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": browserManifest, "main.js": IIFE });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(problems).toEqual([]);
  });

  it("accepts a folder name that differs from the id outside .modelith/plugins/", () => {
    // Arrange（同値クラス: 作業用のフォルダ。名前は自由）
    const plugin = folder({ "modelith-plugin.json": rulesManifest, "rules.yaml": RULES }, { name: "my-checks", inProject: false });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(problems).toEqual([]);
  });

  it("resolves . and .. that stay inside the folder", () => {
    // Arrange（境界値: 一度下りてから戻る）
    const manifest = { ...browserManifest, main: "./dist/../main.js" };
    const plugin = folder({ "modelith-plugin.json": manifest, "main.js": IIFE });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(problems).toEqual([]);
  });

  // --- 異常系: マニフェスト ---

  it("reports a missing manifest", () => {
    // Arrange
    const plugin = folder({});
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(["modelith-plugin.json: error: not found"]);
  });

  it("reports a manifest that is not JSON", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": "{ id: " });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)[0]).toMatch(/^modelith-plugin\.json: error: invalid JSON/);
  });

  it("names an unknown field in the manifest", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": { ...rulesManifest, autor: "x" }, "rules.yaml": RULES });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: unknown field "autor"']);
  });

  it("reports an apiVersion this SDK does not support", () => {
    // Arrange（パターンには合うが、値が違う）
    const plugin = folder({ "modelith-plugin.json": { ...rulesManifest, apiVersion: "1" }, "rules.yaml": RULES });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: apiVersion is "1", but this SDK supports "0"']);
  });

  it("requires the folder name to match the id inside .modelith/plugins/", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": rulesManifest, "rules.yaml": RULES }, { name: "checks", inProject: true });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual([
      'modelith-plugin.json: error: id "com.example.checks" must match the folder name "checks" inside .modelith/plugins/',
    ]);
  });

  it("reports a command id used twice", () => {
    // Arrange
    const manifest = { ...browserManifest, contributes: { commands: [{ id: "run", title: "A" }, { id: "run", title: "B" }] } };
    const plugin = folder({ "modelith-plugin.json": manifest, "main.js": IIFE });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: command id "run" is used more than once']);
  });

  // --- 異常系: main ---

  it("requires main for a browser plugin", () => {
    // Arrange
    const { main: _, ...manifest } = browserManifest;
    const plugin = folder({ "modelith-plugin.json": manifest });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: "main" is required for runtime "browser"']);
  });

  it.each(["../main.js", "/main.js", "C:/main.js", "dist\\main.js"])("rejects main outside the folder: %s", (main: string) => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": { ...browserManifest, main }, "main.js": IIFE });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual([`modelith-plugin.json: error: main "${main}" must be a relative path inside the plugin folder`]);
  });

  it("reports a main file that does not exist", () => {
    // Arrange（ビルドを忘れた）
    const plugin = folder({ "modelith-plugin.json": browserManifest });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: main "main.js" not found']);
  });

  it("reports a browser main that is still an ES module", () => {
    // Arrange（まとめずに tsc の出力をそのまま指した）
    const plugin = folder({ "modelith-plugin.json": browserManifest, "main.js": 'import { connectToHost } from "@modelith/plugin-sdk";\n' });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)[0]).toMatch(/^main\.js: error: uses import\/export/);
  });

  it("does not look for import/export in a wasm-component main", () => {
    // Arrange（同値クラス: browser 以外の実行方式）
    const manifest = { ...browserManifest, runtime: "wasm-component", main: "plugin.wasm" };
    const plugin = folder({ "modelith-plugin.json": manifest, "plugin.wasm": "export" });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(problems).toEqual([]);
  });

  // --- 異常系: ルールのファイル ---

  it("requires at least one rule file for a rules plugin", () => {
    // Arrange（境界値: 0 件）
    const plugin = folder({ "modelith-plugin.json": { ...rulesManifest, contributes: { rules: [] } } });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: a "rules" plugin needs at least one file in contributes.rules']);
  });

  it("reports a rule file that does not exist", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": rulesManifest });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: contributes.rules "rules.yaml" not found']);
  });

  it("reports a rule file that is not YAML", () => {
    // Arrange
    const plugin = folder({ "modelith-plugin.json": rulesManifest, "rules.yaml": "rules: [\n" });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)[0]).toMatch(/^rules\.yaml: error: invalid YAML/);
  });

  it("names an unknown key in a rule", () => {
    // Arrange（よくある綴りの誤り）
    const plugin = folder({ "modelith-plugin.json": rulesManifest, "rules.yaml": RULES.replace("incoming", "incomming") });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['rules.yaml: error: rules.0.require: unknown field "incomming"']);
  });

  it("reports a rule id used in two rule files", () => {
    // Arrange（ID はプラグインの中で一意）
    const manifest = { ...rulesManifest, contributes: { rules: ["a.yaml", "b.yaml"] } };
    const plugin = folder({ "modelith-plugin.json": manifest, "a.yaml": RULES, "b.yaml": RULES });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: error: rule id "req-has-satisfier" is used more than once']);
  });

  it("warns about rule files in a plugin that is not a rules plugin", () => {
    // Arrange
    const manifest = { ...browserManifest, contributes: { rules: ["rules.yaml"] } };
    const plugin = folder({ "modelith-plugin.json": manifest, "main.js": IIFE, "rules.yaml": RULES });
    // Act
    const problems = validatePlugin(plugin, schemas);
    // Assert
    expect(messages(problems)).toEqual(['modelith-plugin.json: warning: contributes.rules is only used by runtime "rules"']);
  });
});

describe("isInsideFolder", () => {
  it.each(["main.js", "./main.js", "dist/main.js", "dist/../main.js"])("accepts %s", (path: string) => {
    // Arrange（正常系。境界値: 一度下りて戻る）
    const candidate = path;
    // Act
    const inside = isInsideFolder(candidate);
    // Assert
    expect(inside).toBe(true);
  });

  it.each(["", ".", "dist/..", "..", "../main.js", "dist/../../main.js", "/main.js", "C:main.js", "dist\\main.js"])(
    "rejects %j",
    (path: string) => {
      // Arrange（異常系。境界値: フォルダそのもの・1 つ外）
      const candidate = path;
      // Act
      const inside = isInsideFolder(candidate);
      // Assert
      expect(inside).toBe(false);
    },
  );
});

describe("looksLikeModule", () => {
  it.each([
    'import { x } from "y";',
    'import x from "y";',
    'import * as x from "y";',
    'import "y";',
    'await import("./y.js");'.replace("await ", ""),
    "export const x = 1;",
    "export { x };",
    "  export default x;",
  ])("finds module syntax in %j", (code: string) => {
    // Arrange
    const script = code;
    // Act
    const found = looksLikeModule(script);
    // Assert
    expect(found).toBe(true);
  });

  it.each([IIFE, 'const label = "import";', "const exporter = 1;", "// import is not a statement here\n"])(
    "does not flag %j",
    (code: string) => {
      // Arrange（行頭の文だけを見る。文字列・識別子・コメントの途中は対象外）
      const script = code;
      // Act
      const found = looksLikeModule(script);
      // Assert
      expect(found).toBe(false);
    },
  );
});

describe("modelith-plugin validate", () => {
  /** パス → 中身 のファイルを持つ偽の入出力。 */
  function io(files: Record<string, string> = {}): Io & { lines: string[] } {
    const lines: string[] = [];
    return { readFile: (path) => files[path], out: (line) => lines.push(line), schemas, lines };
  }

  it("exits 0 and prints a summary for a valid plugin", () => {
    // Arrange
    const fake = io({ "/p/checks/modelith-plugin.json": JSON.stringify(rulesManifest), "/p/checks/rules.yaml": RULES });
    // Act
    const code = run(["validate", "/p/checks"], fake);
    // Assert
    expect([code, fake.lines]).toEqual([0, ["0 error(s), 0 warning(s) in /p/checks"]]);
  });

  it("exits 1 when there is an error", () => {
    // Arrange
    const fake = io({});
    // Act
    const code = run(["validate", "/p/checks"], fake);
    // Assert
    expect([code, fake.lines]).toEqual([1, ["modelith-plugin.json: error: not found", "1 error(s), 0 warning(s) in /p/checks"]]);
  });

  it("exits 0 when there are only warnings", () => {
    // Arrange（境界値: 警告だけ）
    const manifest = { ...browserManifest, contributes: { rules: ["rules.yaml"] } };
    const fake = io({ "/p/tool/modelith-plugin.json": JSON.stringify(manifest), "/p/tool/main.js": IIFE, "/p/tool/rules.yaml": RULES });
    // Act
    const code = run(["validate", "/p/tool"], fake);
    // Assert
    expect(code).toBe(0);
  });

  it("checks the folder name against the id inside .modelith/plugins/", () => {
    // Arrange（パスからプロジェクトの中だと分かる）
    const dir = "/p/kettle/.modelith/plugins/checks";
    const fake = io({ [`${dir}/modelith-plugin.json`]: JSON.stringify(rulesManifest), [`${dir}/rules.yaml`]: RULES });
    // Act
    const code = run(["validate", dir], fake);
    // Assert
    expect([code, fake.lines[0]]).toEqual([
      1,
      'modelith-plugin.json: error: id "com.example.checks" must match the folder name "checks" inside .modelith/plugins/',
    ]);
  });

  it.each([[[]], [["check"]], [["validate", "a", "b"]]])("exits 2 with usage for %j", (args: string[]) => {
    // Arrange（異常系: コマンドが無い・違う・引数が多い）
    const fake = io();
    // Act
    const code = run(args, fake);
    // Assert
    expect([code, fake.lines[0]]).toEqual([2, expect.stringMatching(/^usage: modelith-plugin validate/)]);
  });
});
