#!/usr/bin/env bash
# ハーネスの単一エントリポイント。人間・AI エージェント・CI すべてがこれを実行する。
#   scripts/check.sh          フルチェック
#   scripts/check.sh --fast   高速チェック（Stop フック用）
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

mode="full"
[[ "${1:-}" == "--fast" ]] && mode="fast"

step() { printf '\n==> %s\n' "$*"; }
ts=packages/typescript

step "branch name"
scripts/check-branch-name.sh

# 依存物・ビルド成果物が Git 管理下に入っていないか（guard-git.sh と同じパターン）
step "forbidden files"
FORBIDDEN='(^|/)node_modules/|^target/|^packages/typescript/(dist|schema)/'
if tracked="$(git ls-files | grep -E "$FORBIDDEN")"; then
  echo "ERROR: コミットしてはいけないファイルが Git 管理下にあります:" >&2
  echo "$tracked" | head -20 >&2
  exit 1
fi

step "rust: fmt"
cargo fmt --all -- --check

if [[ "$mode" == "fast" ]]; then
  step "rust: check"
  cargo check --workspace --all-targets --quiet
else
  step "rust: clippy"
  cargo clippy --workspace --all-targets --all-features -- -D warnings
fi

# 生成物（schema/・TS 型）が Rust の型と一致するかもここで検証される
step "rust: test（生成物の最新性・examples の読み込みを含む）"
cargo test --workspace --all-features --quiet

step "typescript: typecheck"
[[ -d $ts/node_modules ]] || npm ci --prefix $ts --no-audit --no-fund
npm run --prefix $ts -s typecheck

if [[ "$mode" == "full" ]]; then
  step "typescript: test（JSON Schema で examples を検証）"
  npm run --prefix $ts -s test
  step "typescript: build"
  npm run --prefix $ts -s build
fi

step "OK ($mode)"
