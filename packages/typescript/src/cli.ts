// `modelith-plugin` コマンドの中身。引数を読み、フォルダのファイルを読んで検査し、結果を書き出す（入出力は io で受け取る）。

import { basename, dirname, join, resolve } from "node:path";
import { formatProblem, type PluginFolder, type Schemas, validatePlugin } from "./validate.js";

/** コマンドが使う入出力。テストでは偽物に差し替える。 */
export interface Io {
  /** ファイルを読む（無い・読めないときは undefined）。 */
  readFile(path: string): string | undefined;
  out(line: string): void;
  schemas: Schemas;
}

const USAGE = "usage: modelith-plugin validate [plugin folder]   (default: the current folder)";

/** コマンドを実行し、終了コードを返す（0: 誤り無し、1: 誤りあり、2: 使い方の誤り）。 */
export function run(args: string[], io: Io): number {
  const [command, folderArg, ...rest] = args;
  if (command !== "validate" || rest.length > 0) {
    io.out(USAGE);
    return 2;
  }
  const path = resolve(folderArg ?? ".");
  const folder: PluginFolder = {
    name: basename(path),
    inProject: basename(dirname(path)) === "plugins" && basename(dirname(dirname(path))) === ".modelith",
    read: (file) => io.readFile(join(path, file)),
  };
  const problems = validatePlugin(folder, io.schemas);
  for (const p of problems) io.out(formatProblem(p));
  const errors = problems.filter((p) => p.severity === "error").length;
  io.out(`${errors} error(s), ${problems.length - errors} warning(s) in ${path}`);
  return errors > 0 ? 1 : 0;
}
