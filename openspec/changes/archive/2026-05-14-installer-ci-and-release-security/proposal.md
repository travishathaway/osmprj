## Why

The project produces platform-specific installer binaries but has no automated test that verifies the installer actually works before a release is cut. Additionally, releases currently ship without checksums or signatures, leaving users who want to verify downloads with no mechanism to do so. Both gaps need closing as the project moves toward a public `curl | bash` install experience.

## What Changes

- Add a `smoke-test.yml` workflow that builds the installer and runs it end-to-end, triggered on push to `main` and manually via `workflow_dispatch` (any branch)
- Add `shellcheck` to `prek.toml` so `scripts/install.sh` is syntax-checked locally and in CI
- Add SHA256 checksum generation to the `publish-release` job in `release.yml`, uploading a `checksums.sha256` file alongside the installers
- Add cosign signing of `checksums.sha256` using GitHub Actions OIDC (keyless, no secret management)

## Capabilities

### New Capabilities

- `installer-smoke-test`: End-to-end CI job that builds the installer on native runners and asserts the installed binary is functional; manually triggerable from any branch via `workflow_dispatch`
- `release-checksums`: SHA256 checksum file generated and uploaded as a release asset for every release
- `release-signing`: Cosign keyless signature of the checksum file, backed by GitHub OIDC, enabling users to cryptographically verify release provenance

### Modified Capabilities

- `shell-linting`: Add `shellcheck` hook to existing `prek.toml` pre-commit config so `scripts/install.sh` is checked on every commit

## Impact

- `.github/workflows/release.yml`: new steps in `publish-release` job for checksum generation and cosign signing
- `.github/workflows/smoke-test.yml`: new workflow file
- `prek.toml`: new `shellcheck` hook entry
- No changes to application code, Python, or Rust sources
- Release artifact list grows by two files: `checksums.sha256` and `checksums.sha256.bundle` (cosign bundle)
- Users verifying downloads need `cosign` installed; instructions to be added to project docs separately
