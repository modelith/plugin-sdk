#!/usr/bin/env bats
# scripts/release-npm.sh の判断（npm_dist_tag・npm_tarball）の単体テスト。

setup() {
  load ../helpers
  # shellcheck source=../../../scripts/release-npm.sh
  source "$BATS_TEST_DIRNAME/../../../scripts/release-npm.sh"
}

@test "npm_dist_tag: 正式版は latest、プレリリース版は next（同値分割）" {
  # Arrange
  local versions=(0.2.0 1.0.0 0.2.0-alpha.1 0.2.0-beta.3 0.2.0-rc.1)
  # Act
  local tags=() v
  for v in "${versions[@]}"; do tags+=("$(npm_dist_tag "$v")"); done
  # Assert
  [ "${tags[*]}" = "latest latest next next next" ]
}

@test "npm_dist_tag: ビルドの印（+）だけの版は正式版として latest（境界値）" {
  # Arrange
  local version=0.2.0+build.5
  # Act
  run npm_dist_tag "$version"
  # Assert
  [ "$output" = latest ]
}

@test "npm_tarball: npm pack と同じ名前を返す（正常系: スコープなし・スコープつき）" {
  # Arrange
  local version=0.2.0-rc.1
  # Act
  local plain scoped
  plain=$(npm_tarball modelith-plugin-sdk "$version")
  scoped=$(npm_tarball @modelith/plugin-sdk "$version")
  # Assert
  [ "$plain $scoped" = "modelith-plugin-sdk-0.2.0-rc.1.tgz modelith-plugin-sdk-0.2.0-rc.1.tgz" ]
}
