# Contributing to the Modelith Plugin SDK

Thank you for helping. Bug reports, questions and pull requests are all welcome.
For anything that changes the contract, please open an issue first: plugins, Modelith and the registry all depend on it.

## Set up

You need:

- Rust stable with `rustfmt` and `clippy` (`rust-toolchain.toml` selects them)
- Node.js, the version in [`.nvmrc`](.nvmrc)
- [ShellCheck](https://www.shellcheck.net/)

```sh
npm ci --prefix packages/typescript --ignore-scripts
npm ci --prefix tests/harness --ignore-scripts
scripts/check.sh
```

`scripts/check.sh` is the one command that humans, agents and CI all run. It goes static checks → unit → integration → system
and stops at the first stage that fails. `scripts/check.sh --fast` runs only the static checks and unit tests.

## How the repository fits together

```
crates/modelith-plugin-protocol/src/*.rs   ← the only definition (Rust types)
        │ UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated
        ├─▶ schema/*.schema.json                     (generated)
        └─▶ packages/typescript/src/generated/*.ts   (generated)
examples/*.json  ← checked by both Rust (serde) and TypeScript (the JSON Schemas)
```

- Never edit generated files by hand. Change the Rust types, regenerate, and commit the result.
  The integration tests fail when a generated file is out of date.
- When you change a type, update `examples/` so that it uses the new field.
- Constants that both languages need, such as id patterns, live in Rust and reach TypeScript through the generated file.

| Task | Command |
| --- | --- |
| Full check (required before a PR) | `scripts/check.sh` |
| Quick check | `scripts/check.sh --fast` |
| Regenerate schemas and TS types | `UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated` |
| TypeScript tests | `npm test --prefix packages/typescript` |
| Harness tests | `tests/harness/node_modules/.bin/bats tests/harness/{unit,integration}` |

## Changing the contract

- `apiVersion` (`API_VERSION`) equals the SDK's major version. Before 1.0.0 it is `"0"` and nothing is guaranteed to stay compatible.
- A change that breaks existing plugins is a breaking change. Breaking changes include adding a required field, removing or
  renaming a field or enum value, and changing what something means. Mark the commit with `!` and record the
  decision as an ADR in [Modelith](https://github.com/modelith/modelith) first.
- Add new things compatibly, as optional fields (`Option` / `#[serde(default)]`).
- Manifests and rule files reject unknown fields (`deny_unknown_fields`). Loosening that also needs an ADR.

The full versioning rules are in Modelith's [versioning guide](https://github.com/modelith/modelith/blob/dev/docs/reference/versioning.md).

## Branches, commits and pull requests

The SDK follows Modelith's [branching strategy](https://github.com/modelith/modelith/blob/dev/docs/development/branching-strategy.md).

- Branch from `dev` and name the branch `<type>/<topic>`, for example `feat/rule-set`.
- Open the pull request against `dev`. CI (`scripts/check-pr-base.sh`) rejects other targets.
- Write commit messages and PR titles as [Conventional Commits](https://www.conventionalcommits.org/):
  `<type>(<scope>): <summary>`, with the scope `protocol`, `schema`, `ts` or `harness`.
- Pull requests to `dev` are squash-merged once CI (`check` and `conventions`) is green and no review comment is open.
- Each release merges `dev` into `main` and tags `vMAJOR.MINOR.PATCH`. The crate and the npm package share the version.

Fill in the [pull request template](.github/pull_request_template.md). Behavior changes need tests: unit tests next to
the code, integration tests in `crates/*/tests/`, and the test design rules in
[`.claude/skills/testing-principles`](.claude/skills/testing-principles/SKILL.md) apply.

## Adding or updating dependencies

Commit lockfiles and install npm dependencies with `npm ci`. The `.npmrc` in each npm project (`ignore-scripts=true`) blocks install scripts.
Versions published less than 7 days ago are not taken in, unless they are urgent security fixes.
CI updates rustup itself before installing the toolchain, and requires rustup 1.29.2 or later.
The full policy is in Modelith's [CONTRIBUTING](https://github.com/modelith/modelith/blob/dev/CONTRIBUTING.md#依存の追加更新).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
