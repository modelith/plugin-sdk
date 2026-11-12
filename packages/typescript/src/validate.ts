// プラグインのフォルダを検査する（判断だけ。ファイルの読み込みは呼ぶ側が PluginFolder で渡す）。
// スキーマで分かる形の誤りに加え、フォルダとマニフェストの食い違い（ID・apiVersion・指すファイル）を見る。
// 要素の種類や正規表現など、モデルの意味にかかわる検査は Modelith のコア（`modelith check`）が行う。

import Ajv2020, { type ErrorObject, type ValidateFunction } from "ajv/dist/2020.js";
import { parse as parseYaml } from "yaml";
import { API_VERSION, type PluginManifest, type RuleSet } from "./generated/protocol.js";

export const MANIFEST_FILE = "modelith-plugin.json";

/** 検査するプラグインのフォルダ。 */
export interface PluginFolder {
  /** フォルダの名前。 */
  name: string;
  /** プロジェクトの `.modelith/plugins/` の中にあるか（あればフォルダの名前を ID と同じにする）。 */
  inProject: boolean;
  /** フォルダからの相対パス（`/` 区切り）のファイルを読む。無ければ undefined。 */
  read(path: string): string | undefined;
}

/** 検査に使う JSON Schema（plugin-sdk の `schema/`）。 */
export interface Schemas {
  manifest: object;
  ruleSet: object;
}

export interface Problem {
  severity: "error" | "warning";
  /** 問題のあるファイル（フォルダからの相対パス）。 */
  file: string;
  message: string;
}

/** プラグインのフォルダを検査し、見つかった問題を並べる（無ければ空）。 */
export function validatePlugin(folder: PluginFolder, schemas: Schemas): Problem[] {
  const ajv = new Ajv2020.default({ strict: false, logger: false, allErrors: true });
  const manifestText = folder.read(MANIFEST_FILE);
  if (manifestText === undefined) return [error(MANIFEST_FILE, "not found")];
  const json = parseJson(manifestText);
  if (!json.ok) return [error(MANIFEST_FILE, `invalid JSON: ${json.message}`)];
  const schemaErrors = check(ajv.compile(schemas.manifest), json.value);
  if (schemaErrors.length > 0) return schemaErrors.map((m) => error(MANIFEST_FILE, m));
  const manifest = json.value as PluginManifest;
  return [
    ...checkIdentity(folder, manifest),
    ...checkContributedIds(manifest),
    ...checkEntry(folder, manifest),
    ...checkRules(folder, manifest, ajv.compile(schemas.ruleSet)),
  ];
}

/** 問題を 1 件 1 行の文にする（例: `rules.yaml: error: ...`）。 */
export function formatProblem(p: Problem): string {
  return `${p.file}: ${p.severity}: ${p.message}`;
}

function checkIdentity(folder: PluginFolder, manifest: PluginManifest): Problem[] {
  const problems: Problem[] = [];
  if (manifest.apiVersion !== API_VERSION) {
    problems.push(error(MANIFEST_FILE, `apiVersion is "${manifest.apiVersion}", but this SDK supports "${API_VERSION}"`));
  }
  if (folder.inProject && folder.name !== manifest.id) {
    problems.push(error(MANIFEST_FILE, `id "${manifest.id}" must match the folder name "${folder.name}" inside .modelith/plugins/`));
  }
  return problems;
}

function checkContributedIds(manifest: PluginManifest): Problem[] {
  const c = manifest.contributes ?? {};
  return [
    ...duplicates((c.commands ?? []).map((x) => x.id)).map((id) => `command id "${id}"`),
    ...duplicates((c.importers ?? []).map((x) => x.id)).map((id) => `importer id "${id}"`),
    ...duplicates((c.exporters ?? []).map((x) => x.id)).map((id) => `exporter id "${id}"`),
  ].map((what) => error(MANIFEST_FILE, `${what} is used more than once`));
}

/** `main` が要る実行方式なら、それがフォルダの中にあり、動かせる形かを見る。 */
function checkEntry(folder: PluginFolder, manifest: PluginManifest): Problem[] {
  if (manifest.runtime === "rules") return [];
  if (manifest.main === undefined) return [error(MANIFEST_FILE, `"main" is required for runtime "${manifest.runtime}"`)];
  const file = referencedFile(folder, "main", manifest.main);
  if (!file.ok) return [file.problem];
  if (manifest.runtime === "browser" && looksLikeModule(file.text)) {
    return [
      error(
        manifest.main,
        "uses import/export: Modelith runs a browser plugin as a classic script, so bundle it into one file (e.g. esbuild --bundle --format=iife)",
      ),
    ];
  }
  return [];
}

function checkRules(folder: PluginFolder, manifest: PluginManifest, ruleSetSchema: ValidateFunction): Problem[] {
  const paths = manifest.contributes?.rules ?? [];
  if (manifest.runtime !== "rules") {
    return paths.length > 0 ? [warning(MANIFEST_FILE, `contributes.rules is only used by runtime "rules"`)] : [];
  }
  if (paths.length === 0) return [error(MANIFEST_FILE, `a "rules" plugin needs at least one file in contributes.rules`)];
  const problems: Problem[] = [];
  const ids: string[] = [];
  for (const path of paths) {
    const file = referencedFile(folder, "contributes.rules", path);
    if (!file.ok) {
      problems.push(file.problem);
      continue;
    }
    const yaml = parseYamlText(file.text);
    if (!yaml.ok) {
      problems.push(error(path, `invalid YAML: ${yaml.message}`));
      continue;
    }
    const schemaErrors = check(ruleSetSchema, yaml.value);
    problems.push(...schemaErrors.map((m) => error(path, m)));
    if (schemaErrors.length === 0) ids.push(...(yaml.value as RuleSet).rules.map((r) => r.id));
  }
  return [...problems, ...duplicates(ids).map((id) => error(MANIFEST_FILE, `rule id "${id}" is used more than once`))];
}

type FileResult = { ok: true; text: string } | { ok: false; problem: Problem };

/** マニフェストが指すファイルを読む。フォルダの外を指すもの・無いものは問題にする。 */
function referencedFile(folder: PluginFolder, field: string, path: string): FileResult {
  if (!isInsideFolder(path)) return { ok: false, problem: error(MANIFEST_FILE, `${field} "${path}" must be a relative path inside the plugin folder`) };
  const text = folder.read(normalize(path));
  if (text === undefined) return { ok: false, problem: error(MANIFEST_FILE, `${field} "${path}" not found`) };
  return { ok: true, text };
}

/** フォルダからの相対パスで、フォルダの外に出ないか（絶対パス・ドライブ名・`\`・外へ出る `..` は不可）。 */
export function isInsideFolder(path: string): boolean {
  if (path === "" || path.startsWith("/") || path.includes("\\") || /^[A-Za-z]:/.test(path)) return false;
  let depth = 0;
  for (const segment of path.split("/")) {
    if (segment === "..") depth -= 1;
    else if (segment !== "" && segment !== ".") depth += 1;
    if (depth < 0) return false;
  }
  return depth > 0;
}

function normalize(path: string): string {
  const out: string[] = [];
  for (const segment of path.split("/")) {
    if (segment === "..") out.pop();
    else if (segment !== "" && segment !== ".") out.push(segment);
  }
  return out.join("/");
}

/** 行頭に import / export の文があるか（モジュール形式のままのスクリプトを見つける。`import(...)` も読み込めないので含める）。 */
export function looksLikeModule(code: string): boolean {
  return /^[ \t]*(import[\s{*"'(]|export[\s{*])/m.test(code);
}

function duplicates(values: string[]): string[] {
  const seen = new Set<string>();
  const repeated = new Set<string>();
  for (const v of values) (seen.has(v) ? repeated : seen).add(v);
  return [...repeated];
}

function check(validate: ValidateFunction, value: unknown): string[] {
  if (validate(value)) return [];
  return (validate.errors ?? []).map(describe);
}

function describe(e: ErrorObject): string {
  const at = e.instancePath === "" ? "" : `${e.instancePath.slice(1).replaceAll("/", ".")}: `;
  if (e.keyword === "additionalProperties") return `${at}unknown field "${String(e.params.additionalProperty)}"`;
  return `${at}${e.message ?? "is invalid"}`;
}

type Parsed = { ok: true; value: unknown } | { ok: false; message: string };

function parseJson(text: string): Parsed {
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (e) {
    return { ok: false, message: (e as Error).message };
  }
}

function parseYamlText(text: string): Parsed {
  try {
    return { ok: true, value: parseYaml(text) };
  } catch (e) {
    return { ok: false, message: (e as Error).message };
  }
}

const error = (file: string, message: string): Problem => ({ severity: "error", file, message });
const warning = (file: string, message: string): Problem => ({ severity: "warning", file, message });
