#!/usr/bin/env bats
# scripts/release-draft.sh の判断（changelog_dated）の単体テスト。

setup() {
  load ../helpers
  # shellcheck source=../../../scripts/release-draft.sh
  source "$BATS_TEST_DIRNAME/../../../scripts/release-draft.sh"
  CHANGELOG=$'# Changelog\n\n## [Unreleased]\n\n- next\n\n## [0.2.0-rc.1] - 2026-12-01\n\n### Added\n\n- rc\n\n## [0.1.0] - 2026-11-19\n\nThe first release.\n\n[0.1.0]: https://example.com'
}

@test "changelog_dated: 日付のある版は 0、Unreleased・無い版・日付でない見出しは 1（同値分割）" {
  # Arrange
  local undated=$'## [0.3.0] - Unreleased' odd=$'## [0.3.0] - soon'
  # Act
  local dated rc unreleased missing other
  dated=$(status_of_stdin "$CHANGELOG" changelog_dated 0.1.0)
  rc=$(status_of_stdin "$CHANGELOG" changelog_dated 0.2.0-rc.1)
  unreleased=$(status_of_stdin "$undated" changelog_dated 0.3.0)
  missing=$(status_of_stdin "$CHANGELOG" changelog_dated 9.9.9)
  other=$(status_of_stdin "$odd" changelog_dated 0.3.0)
  # Assert
  [ "$dated $rc $unreleased $missing $other" = "0 0 1 1 1" ]
}
