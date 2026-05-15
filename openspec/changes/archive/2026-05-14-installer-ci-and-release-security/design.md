## Context

The project builds platform-specific installer binaries (pixi-pack-generated self-extracting shell scripts) for `linux-64`, `linux-aarch64`, and `osx-arm64`. These are uploaded to GitHub Releases and fetched by `scripts/install.sh`, which users pipe directly into bash. Currently:

- No CI job verifies the installer actually runs after it is built
- Releases ship no checksums or signatures
- `scripts/install.sh` has no syntax checking in the pre-commit pipeline

The workflow already contains a `publish-release` job that uploads assets via `gh release upload`. The pre-commit tool is `prek` (configured in `prek.toml`). GitHub Actions OIDC is available in all workflow jobs.

## Goals / Non-Goals

**Goals:**
- Verify the end-to-end installer flow on every push to `main` and on demand from any branch
- Give security-conscious users a way to verify release assets before running them
- Catch shell syntax errors in `scripts/install.sh` at commit time

**Non-Goals:**
- Cross-platform smoke testing (only native runners; no QEMU emulation)
- Reproducible builds
- Signing individual installer binaries (signing the checksum file is sufficient)
- Automating user-facing verification documentation (docs update is out of scope)

## Decisions

### D1: Smoke test triggers — `push: main` + `workflow_dispatch`

The smoke test workflow uses two triggers:
```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
```

`workflow_dispatch` allows manual runs from any branch via the GitHub Actions UI, which is the primary mechanism for testing installer changes on feature branches before merging. No inputs are required — the workflow always runs against the branch it is dispatched on.

Alternative considered: triggering on `pull_request`. Rejected because building the full conda package + pixi-pack environment on every PR would be slow and expensive. Merging to main is the right gate.

### D2: Smoke test scope — build + install + assert binary works

Each matrix runner builds the installer natively (same steps as `release.yml`) then runs it:
```bash
./osmprj-<platform>-installer.sh --output-directory /tmp/osmprj-test
/tmp/osmprj-test/env/bin/osmprj --version
```

The assertion is intentionally minimal: the binary exists and exits 0. This validates the full pixi-pack environment unpacks correctly and the binary is executable.

Alternative considered: testing `scripts/install.sh` end-to-end (the curl wrapper). Rejected because it requires a live release asset URL, creating a chicken-and-egg dependency. Testing the installer `.sh` directly is equivalent — `scripts/install.sh` is a trivial platform-detection + download shim.

### D3: Smoke test matrix — native runners only, no aarch64

```
linux-64    → ubuntu-latest
osx-arm64   → macos-14
```

`linux-aarch64` is excluded from the smoke test because `ubuntu-24.04-arm` runners are slower and less available. The `linux-aarch64` build is already validated in `release.yml` on every actual release. The smoke test covers the two most common developer platforms.

### D4: Checksums — `sha256sum` in `publish-release`, single file

A single `checksums.sha256` file is generated in the `publish-release` job after all artifacts are downloaded:
```bash
sha256sum installers/*.sh > checksums.sha256
```

This is done in the existing `publish-release` job rather than in each matrix build job to avoid coordination complexity. The checksum file covers all three platform installers and `install.sh`.

### D5: Signing — cosign keyless via GitHub OIDC

The `checksums.sha256` file is signed with `cosign sign-blob` using the GitHub Actions OIDC token:
```bash
cosign sign-blob \
  --yes \
  --bundle checksums.sha256.bundle \
  checksums.sha256
```

This produces a `checksums.sha256.bundle` file uploaded alongside the checksum. Verification requires `cosign verify-blob` with the workflow's identity and OIDC issuer — no secrets or key management needed.

The `publish-release` job needs `id-token: write` permission added.

Alternative considered: GPG signing. Rejected because it requires managing a long-lived signing key as a GitHub secret, and users must obtain and trust the public key out-of-band. Cosign's keyless model ties the signature to the specific GitHub Actions workflow identity, which is both easier to manage and easier for users to verify.

### D6: shellcheck — added to `prek.toml`

```toml
[[repos]]
repo = "https://github.com/shellcheck-py/shellcheck-py"
rev = "v0.10.0.1"
hooks = [
  { id = "shellcheck" }
]
```

This runs shellcheck on every commit via prek, and is also covered by the existing `pre-commit` CI job in `ci.yml`. No new CI job is needed.

Alternative considered: adding shellcheck as a separate `ci.yml` step. Rejected because prek already handles pre-commit checks in CI, and adding a separate step would duplicate the concern.

## Risks / Trade-offs

- **Slow smoke test builds**: Building the full conda package + pixi-pack environment takes several minutes. → Acceptable; the smoke test runs post-merge, not blocking PRs.
- **ubuntu-24.04-arm availability**: ARM runners on GitHub Actions can queue. → Mitigated by excluding aarch64 from smoke tests.
- **cosign binary availability on runners**: `cosign` is not pre-installed on GitHub-hosted runners. → Install via `sigstore/cosign-installer` action before the signing step.
- **Checksum generated post-download**: If an artifact download fails silently, the checksum file would be incomplete. → `actions/download-artifact` fails loudly; no special handling needed.
- **prek version pinning**: shellcheck-py rev must stay current. → Treated the same as other prek hooks; update as needed.

## Migration Plan

1. Add shellcheck hook to `prek.toml` — no workflow changes needed, takes effect immediately
2. Add `smoke-test.yml` — triggers on next push to `main`
3. Update `release.yml` `publish-release` job — takes effect on next release

No rollback complexity. All changes are additive; removing a workflow file or reverting `prek.toml` is the full rollback for any step.

## Open Questions

- None. All decisions are resolved based on the exploration discussion.
