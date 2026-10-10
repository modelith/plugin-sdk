#!/usr/bin/env bash
# PreToolUse(Edit|Write): 生成物と lock ファイルをエージェントが直接書き換えないようにする。
set -euo pipefail

file="$(jq -r '.tool_input.file_path // .tool_input.notebook_path // ""')"
rel="${file#"${CLAUDE_PROJECT_DIR:-$PWD}"/}"

case "$rel" in
  schema/* | packages/typescript/src/generated/*)
    msg="生成物です。crates/modelith-plugin-protocol の型を変更し、'UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated' で再生成してください。" ;;
  */package-lock.json | package-lock.json | Cargo.lock)
    msg="lock ファイルは手で編集せず、npm / cargo コマンドで更新してください。" ;;
  *) exit 0 ;;
esac
echo "BLOCKED: $msg" >&2
exit 2
