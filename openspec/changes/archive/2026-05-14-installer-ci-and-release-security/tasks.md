## 1. Shell Linting

- [x] 1.1 Add `shellcheck-py` repo and hook to `prek.toml`
- [x] 1.2 Run shellcheck locally against `scripts/install.sh` and fix any violations

## 2. Smoke Test Workflow

- [x] 2.1 Create `.github/workflows/smoke-test.yml` with `push: branches: [main]` and `workflow_dispatch` triggers
- [x] 2.2 Add build matrix for `linux-64` (`ubuntu-latest`) and `osx-arm64` (`macos-14`)
- [x] 2.3 Add steps to each matrix job: checkout, setup-pixi (rust + pack envs), build Rust binary, build conda package, pack environment, rename installer
- [x] 2.4 Add step to run the installer: `./osmprj-<platform>-installer.sh --output-directory /tmp/osmprj-test`
- [x] 2.5 Add assertion step: `/tmp/osmprj-test/env/bin/osmprj --version`

## 3. Release Checksums

- [x] 3.1 In the `publish-release` job in `release.yml`, add a step after artifact download to generate `checksums.sha256` with `sha256sum installers/*.sh scripts/install.sh`
- [x] 3.2 Add `checksums.sha256` to the `gh release upload` command

## 4. Cosign Signing

- [x] 4.1 Add `id-token: write` permission to the `publish-release` job in `release.yml`
- [x] 4.2 Add `sigstore/cosign-installer` action step before signing in `publish-release`
- [x] 4.3 Add `cosign sign-blob --yes --bundle checksums.sha256.bundle checksums.sha256` step
- [x] 4.4 Add `checksums.sha256.bundle` to the `gh release upload` command
