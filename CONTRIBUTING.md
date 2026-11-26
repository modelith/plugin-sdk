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
- The code examples in the READMEs and `docs/` are tested: TypeScript examples are type-checked, JSON manifests and YAML rules
  are checked against the schemas, and the browser plugin tutorial is run against a simulated host. Keep them complete enough to run.

| Task | Command |
| --- | --- |
| Full check (required before a PR) | `scripts/check.sh` |
| Quick check | `scripts/check.sh --fast` |
| Regenerate schemas and TS types | `UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated` |
| TypeScript tests | `npm test --prefix packages/typescript` |
| Check the code examples in the READMEs and tutorials | `npm run test:integration --prefix packages/typescript` |
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
  See [Releasing](#releasing).

Fill in the [pull request template](.github/pull_request_template.md). Behavior changes need tests: unit tests next to
the code, integration tests in `crates/*/tests/`, and the test design rules in
[`.claude/skills/testing-principles`](.claude/skills/testing-principles/SKILL.md) apply.

## Releasing

The crate and the npm package are released together, with the same version, from `main`, the same way as Modelith itself
([releasing.md](https://github.com/modelith/modelith/blob/dev/docs/development/releasing.md)):
a release draft is created automatically, and publishing the draft creates the tag and starts the release.

- [Release draft](.github/workflows/release-draft.yml): when a version's release notes (`docs/releases/v<version>.md`) reach `main`,
  it creates a draft GitHub Release `v<version>` targeting that commit, with the notes as its body
  (`scripts/release-draft.sh`). Run it by hand with a version to create the draft or replace its notes.
- [Release](.github/workflows/release.yml), started by publishing the draft:
  1. **verify**: the tag must be on `main` and match the version of both packages (`scripts/check-versions.sh <tag>`),
     and `scripts/check.sh` must pass.
  2. **publish**: runs in the `release` environment, so it waits until a maintainer approves it.
     It publishes the crate to crates.io (skipped if that version is already there) and attaches the TypeScript package
     (`npm pack`) to the release (replaced if already attached), so a failed run can simply be re-run.
  3. **npm**: stages the same `.tgz` on npm as `modelith-plugin-sdk` (`scripts/release-npm.sh`, `npm stage publish`;
     Modelith's ADR-0028). Staging does not publish: a maintainer reviews the staged version on
     <https://www.npmjs.com/package/modelith-plugin-sdk> and approves it with two-factor authentication
     (or `npm stage list modelith-plugin-sdk`, then `npm stage approve <ID>` / `npm stage reject <ID>`).
     A pre-release gets the dist-tag `next`, a release `latest`. The workflow holds no npm token.
     A version can be staged only once: reject a staged version before re-running the job.

Steps:

1. On a branch from `dev` (`chore/release-v<version>`), set the version in `crates/modelith-plugin-protocol/Cargo.toml`
   and `packages/typescript/package.json` (and their lockfiles), and move the `Unreleased` entries in `CHANGELOG.md` under the new version
   with today's date. `scripts/check.sh` checks that both versions match. Merge the PR into `dev`.
   Write the release notes for users in `docs/releases/v<version>.md` (what changed for plugin authors, how to install,
   anything to watch out for), and link to `CHANGELOG.md` for the full list.
2. Open a PR from `dev` to `main`. **Merging it needs a maintainer's explicit approval**, and it is merged with a merge commit.
   Merging creates the release draft.
3. A maintainer checks the draft and presses **Publish release**. This creates the tag `v<version>` on `main`.
4. Approve the `publish` job when it asks, and check the new version on crates.io and the GitHub Release.
   Then approve the staged version on npm, and check `npm view modelith-plugin-sdk version`.
5. Merge `main` back into `dev` with a merge commit.

A pre-release version (`0.2.0-rc.1`) gets a GitHub Release marked as a pre-release.

### One-time setup

A maintainer sets these up once, in the repository settings:

- **Settings → Environments → New environment `release`**
  - Required reviewers: the maintainers who approve each release.
  - Deployment branches and tags: tags matching `v*` only.
  - No crates.io or npm token is stored: the workflow reaches both through trusted publishing.
- **crates.io** (an owner of `modelith-plugin-protocol` does this; never put a token in the repository, an issue or a chat):
  1. On crates.io, open the crate's Settings → Trusted Publishing and add a GitHub publisher:
     repository owner `modelith`, repository `plugin-sdk`, workflow `release.yml`, environment `release`.
  2. Publish one release through the workflow to check that it works.
  3. Then delete the environment secret `CARGO_REGISTRY_TOKEN` if it is still there, revoke that token on crates.io
     (Account Settings → API Tokens). If the crate's settings offer an option to allow only trusted publishing, turn it on.
- **npm** (npm 11.15.0 or later; never put a token or a one-time password in the repository, an issue or a chat).
  The package is owned by the npm account used for publishing, the same one as `modelith-cli`. Version 0.2.0 was published by hand,
  from the `.tgz` the workflow attached to the release; every later version goes through the workflow.
  1. Register the trusted publisher, allowing staging only (do not add `--allow-publish`), on npmjs.com
     (the package's Settings → Trusted Publisher) or from the command line:

     ```sh
     npm trust github modelith-plugin-sdk --repo modelith/plugin-sdk --file release.yml --allow-stage-publish
     ```

     A new registration is "Pending validation": npm validates it when the workflow publishes for the first time, and gives
     about two days to do so. A publish by hand does not validate it. So register it shortly before a release, and check
     after that release that the status is no longer pending. If the time runs out, register it again before the next release.
  2. In the package's settings on npmjs.com, choose "Require two-factor authentication and disallow bypass 2fa tokens".
  3. In the repository's Settings → Secrets and variables → Actions → Variables, add `NPM_STAGE_PUBLISH` = `true`.
     Until then the workflow skips the npm job.

  Registering a trusted publisher for a name that is not on npm yet creates a placeholder version `0.0.0-stage`.
  It is not a release; `latest` moves to the first real version.

## Adding or updating dependencies

Commit lockfiles and install npm dependencies with `npm ci`. The `.npmrc` in each npm project (`ignore-scripts=true`) blocks install scripts.
Versions published less than 7 days ago are not taken in, unless they are urgent security fixes.
CI updates rustup itself before installing the toolchain, and requires rustup 1.29.2 or later.
The full policy is in Modelith's [CONTRIBUTING](https://github.com/modelith/modelith/blob/dev/CONTRIBUTING.md#依存の追加更新).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
