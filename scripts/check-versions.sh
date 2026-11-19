#!/usr/bin/env bash
# crate と npm パッケージの版がそろっているか、リリースのタグがその版を指しているかを検証する。
# （apiVersion と crate の MAJOR の一致は、crate の単体テストが検証する）
#   scripts/check-versions.sh          crate と npm の版の一致だけを見る（check.sh）
#   scripts/check-versions.sh v0.1.0   タグもその版か見る（リリースのワークフロー）
set -euo pipefail

# 判断だけを行う。0: そろっている、1: 理由を 1 行出す。
# 例: versions_match 0.1.0 0.1.0 v0.1.0 → 0、versions_match 0.1.0 0.2.0 "" → 1
versions_match() {
  local crate=$1 npm=$2 tag=${3:-}
  if [[ -z $crate || -z $npm ]]; then
    echo "版を読み取れませんでした（crate: '$crate'、npm: '$npm'）。"
    return 1
  fi
  if [[ $crate != "$npm" ]]; then
    echo "crate（$crate）と npm パッケージ（$npm）の版が違います。同じ版にしてください。"
    return 1
  fi
  if [[ -n $tag && $tag != "v$crate" ]]; then
    echo "タグ $tag は、crate と npm パッケージの版（$crate）と合いません。v$crate を打ってください。"
    return 1
  fi
}

main() {
  cd "$(git rev-parse --show-toplevel)"
  local crate npm
  crate=$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/modelith-plugin-protocol/Cargo.toml | head -n 1)
  npm=$(node -p 'require("./packages/typescript/package.json").version' 2>/dev/null || true)
  local reason
  if reason=$(versions_match "$crate" "$npm" "${1:-}"); then
    echo "versions OK: $crate${1:+ ($1)}"
  else
    echo "ERROR: $reason" >&2
    exit 1
  fi
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then main "$@"; fi
