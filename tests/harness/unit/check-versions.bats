#!/usr/bin/env bats
# scripts/check-versions.sh の判断（versions_match）の単体テスト。

setup() {
  load ../helpers
  # shellcheck source=../../../scripts/check-versions.sh
  source "$BATS_TEST_DIRNAME/../../../scripts/check-versions.sh"
}

@test "versions_match: crate と npm が同じなら通す（正常系）" {
  # Arrange
  local crate=0.1.0 npm=0.1.0
  # Act
  run versions_match "$crate" "$npm"
  # Assert
  [ "$status" -eq 0 ]
}

@test "versions_match: タグが同じ版を指していれば通す（正常系）" {
  # Arrange（プレリリースの印も含めて一致）
  local version=0.2.0-rc.1
  # Act
  run versions_match "$version" "$version" "v$version"
  # Assert
  [ "$status" -eq 0 ]
}

@test "versions_match: crate と npm の版が違えば止める（異常系）" {
  # Arrange
  local crate=0.1.0 npm=0.1.1
  # Act
  run versions_match "$crate" "$npm"
  # Assert
  [ "$status" -eq 1 ]
  [[ $output == *"0.1.0"*"0.1.1"* ]]
}

@test "versions_match: タグが版と違えば止める（異常系: v の無いタグ・別の版）" {
  # Arrange
  local tags=("0.1.0" "v0.1.1" "v0.1.0-rc.1")
  # Act
  local results=() t
  for t in "${tags[@]}"; do results+=("$(status_of versions_match 0.1.0 0.1.0 "$t")"); done
  # Assert
  [ "${results[*]}" = "1 1 1" ]
}

@test "versions_match: 版を読み取れなければ止める（境界値: 空）" {
  # Arrange
  local crate="" npm=0.1.0
  # Act
  run versions_match "$crate" "$npm"
  # Assert
  [ "$status" -eq 1 ]
}
