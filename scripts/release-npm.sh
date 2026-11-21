#!/usr/bin/env bash
# リリースに添付した TypeScript のパッケージ（.tgz）を、npm のレジストリに「置く」（npm stage publish。modelith の ADR-0028 の 3）。
# 置いただけでは公開されない。メンテナが npm で中身を見て、2 段階認証で承認すると公開される。
# .github/workflows/release.yml が、添付のあとに実行する。認証は GitHub Actions と npm の結び付け（Trusted Publishing）で行い、鍵は使わない。
#   scripts/release-npm.sh <タグ>   例: scripts/release-npm.sh v0.2.0
# 添付したものと同じファイルを置くため、組み立て直さずに、リリースから取ってくる。
# modelith の scripts/release-cli-npm.sh と同じ流れ。
set -euo pipefail

# npm に付ける印（dist-tag）。プレリリース版は next、正式版は latest（`npm install` が入れるのは latest）。
# 例: npm_dist_tag 0.2.0 → latest、npm_dist_tag 0.2.0-rc.1 → next
npm_dist_tag() {
  local version=$1
  if [[ $version == *-* ]]; then echo next; else echo latest; fi
}

# npm pack が付けるファイルの名前。例: npm_tarball modelith-plugin-sdk 0.2.0 → modelith-plugin-sdk-0.2.0.tgz
# スコープつきの名前（@scope/name）は、npm pack と同じく scope-name にする。
npm_tarball() {
  local name=${1#@} version=$2
  echo "${name//\//-}-$version.tgz"
}

main() {
  cd "$(git rev-parse --show-toplevel)"
  local tag=${1:?使い方: scripts/release-npm.sh <タグ>}
  scripts/check-versions.sh "$tag"
  local name version tarball out dist_tag
  name=$(node -p 'require("./packages/typescript/package.json").name')
  version=${tag#v}
  # npm view は、版が無いと何も出さない（npm の版によっては、それでも 0 で終わる）ので、出力で確かめる
  if [[ $(npm view "$name@$version" version 2>/dev/null) == "$version" ]]; then
    echo "$name@$version は、もう npm に公開されています"
    return 0
  fi
  tarball=$(npm_tarball "$name" "$version")
  out=$(mktemp -d)
  gh release download "$tag" --pattern "$tarball" --dir "$out" --repo "${GITHUB_REPOSITORY:?}"
  [[ -f $out/$tarball ]] || { echo "ERROR: リリース $tag に $tarball がありません" >&2; exit 1; }
  dist_tag=$(npm_dist_tag "$version")
  npm stage publish "$out/$tarball" --tag "$dist_tag"
  echo "$name@$version を、印 $dist_tag で npm に置きました。npm（https://www.npmjs.com/package/$name）で承認すると公開されます"
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then main "$@"; fi
