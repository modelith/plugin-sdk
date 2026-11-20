#!/usr/bin/env bash
# リリースの下書きを作る（CONTRIBUTING.md の「Releasing」）。.github/workflows/release-draft.yml が実行する。
#   scripts/release-draft.sh --auto   main に push されたとき。crate の版のリリースノートがあり、
#                                     その版のリリースがまだ無ければ下書きを作る。それ以外は何もしない
#   scripts/release-draft.sh <版>     手で起動したとき。下書きが無ければ作り、あれば本文と対象のコミット（下書きのとき）を差し替える
#
# 本文は docs/releases/v<版>.md（利用者向けのリリースノート。細かな変更の一覧は CHANGELOG.md）。
# 版を出す前に、CHANGELOG.md のその版の節に日付が入っていることも確かめる。
# タグは、メンテナが下書きを公開（Publish）したときに作られ、
# 公開すると .github/workflows/release.yml が crate を crates.io に出し、TS パッケージを添付する。
# modelith の scripts/release-draft.sh と同じ流れ。
set -euo pipefail

# 標準入力の CHANGELOG で、その版の見出しに日付（`## [<版>] - YYYY-MM-DD`）があるか。0: ある、1: 無い。
# 日付の無い（`Unreleased` の）版は、まだ出さない。
changelog_dated() {
  local version=$1 line
  line=$(grep -F "## [$version] - " || true)
  [[ $line =~ ^\#\#\ \[[^]]+\]\ -\ [0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]
}

main() {
  cd "$(git rev-parse --show-toplevel)"
  local arg=${1:?使い方: scripts/release-draft.sh --auto | <版>}
  local version=$arg
  [[ $arg == --auto ]] && version=$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/modelith-plugin-protocol/Cargo.toml | head -n 1)
  local tag="v$version"
  local notes="docs/releases/$tag.md"

  if [[ ! -f $notes ]]; then
    [[ $arg == --auto ]] && { echo "$notes が無いので、下書きは作りません（版を出す PR ではない）"; exit 0; }
    echo "ERROR: リリースノート $notes がありません。" >&2
    exit 1
  fi
  if ! changelog_dated "$version" <CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md に '## [$version] - YYYY-MM-DD' の見出しがありません。" >&2
    exit 1
  fi
  local branch=${GITHUB_REF_NAME:-$(git branch --show-current)}
  if [[ $arg == --auto && $branch != main ]]; then
    echo "リリースは main から出すので、$branch では下書きを作りません"
    exit 0
  fi
  scripts/check-versions.sh "$tag"

  local existing
  existing=$(gh release list --limit 100 --json tagName --jq ".[] | select(.tagName == \"$tag\") | .tagName")
  if [[ -n $existing && $arg == --auto ]]; then
    echo "リリース $tag は既にあります。本文を差し替えるときは、版を渡して手で起動してください"
    exit 0
  elif [[ -n $existing ]]; then
    gh release edit "$tag" --notes-file "$notes" --target "$(git rev-parse HEAD)"
    echo "リリース $tag の本文を差し替えました"
  else
    local prerelease=()
    [[ $version == *-* ]] && prerelease=(--prerelease)
    gh release create "$tag" --draft "${prerelease[@]}" --target "$(git rev-parse HEAD)" --title "$tag" --notes-file "$notes"
    echo "リリース $tag の下書きを作りました（対象: $(git rev-parse --short HEAD)）"
  fi
  # 下書きは、タグからは引けないので一覧から探す
  gh api "repos/{owner}/{repo}/releases" --jq ".[] | select(.tag_name == \"$tag\") | \"下書き・リリースのページ: \" + .html_url"
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then main "$@"; fi
