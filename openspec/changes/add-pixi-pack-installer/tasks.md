## 1. Update pixi.toml

- [x] 1.1 Add `[feature.pack.dependencies]` section with `rattler-build = "*"` and `pixi-pack = "*"`
- [x] 1.2 Add `pack = { features = ["pack"], no-default-feature = true }` to the `[environments]` table
- [x] 1.3 Run `pixi install --environment pack` to solve and update `pixi.lock` with the new environment
- [x] 1.4 Verify `pixi run --environment pack rattler-build --version` and `pixi run --environment pack pixi-pack --version` both succeed

## 2. Create scripts/install.sh

- [x] 2.1 Create `scripts/install.sh` with a shebang `#!/usr/bin/env bash` and `set -euo pipefail`
- [x] 2.2 Add platform detection: map `uname -s` + `uname -m` to `linux-64` or `osx-arm64`; print an unsupported message and exit 1 for any other combination
- [x] 2.3 Set `INSTALL_DIR="${OSMPRJ_INSTALL_DIR:-$HOME/.local/osmprj}"` and `RELEASE_BASE` pointing to `https://github.com/travishathaway/osmprj/releases/latest/download`
- [x] 2.4 Download the platform-specific installer with `curl -fsSL` to `/tmp/osmprj-installer.sh`, `chmod +x`, then run it with `--output-directory "$INSTALL_DIR"`
- [x] 2.5 Add shell rc detection: check `$SHELL` for `zsh` → `~/.zshrc`, `bash` → `~/.bashrc`, fallback to `~/.profile`
- [x] 2.6 Add idempotent rc file patching: check for an existing `# osmprj` marker before appending the `export PATH` and `export OSMPRJ_THEME_PATH` lines
- [x] 2.7 Remove `/tmp/osmprj-installer.sh` after install
- [x] 2.8 Print a success message showing the install directory and instructions to restart the shell or source the rc file
- [x] 2.9 `chmod +x scripts/install.sh` and manually verify the platform detection logic runs correctly on the current machine

## 3. Create .github/workflows/release.yml

- [x] 3.1 Create `.github/workflows/release.yml` with `on: release: types: [published]`
- [x] 3.2 Add `build-installer` job with a matrix of `{platform: linux-64, runner: ubuntu-latest}` and `{platform: osx-arm64, runner: macos-14}`
- [x] 3.3 Add checkout step (`actions/checkout@v4`)
- [x] 3.4 Add `prefix-dev/setup-pixi@v0.8.5` step with `environments: rust pack` and `cache: true`
- [x] 3.5 Add rattler-build step: `pixi run --environment pack rattler-build build -r recipe/recipe.yaml --output-dir output/`
- [x] 3.6 Add pixi-pack step: `pixi run --environment pack pixi-pack pack --environment default --platform ${{ matrix.platform }} --inject "output/${{ matrix.platform }}/*.conda" --create-executable pixi.toml`
- [x] 3.7 Add rename step: `mv environment.sh osmprj-${{ matrix.platform }}-installer.sh`
- [x] 3.8 Add `actions/upload-artifact@v4` step with `name: installer-${{ matrix.platform }}` and `path: osmprj-*-installer.sh`
- [x] 3.9 Add `publish-release` job that `needs: build-installer`, runs on `ubuntu-latest`
- [x] 3.10 Add checkout step to `publish-release`
- [x] 3.11 Add `actions/download-artifact@v4` step with `path: installers/`, `pattern: installer-*`, `merge-multiple: true`
- [x] 3.12 Add `gh release upload` step using `${{ github.ref_name }}` to upload `installers/osmprj-linux-64-installer.sh`, `installers/osmprj-osx-arm64-installer.sh`, and `scripts/install.sh`; set `env: GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`

## 4. Verification

- [x] 4.1 Validate `release.yml` syntax using `actionlint` (available via `pixi run --environment dev` or install globally) or push to a test branch and check the Actions tab
- [ ] 4.2 Run `scripts/install.sh` on a clean Linux environment (a Docker container or VM) and confirm `osmprj` appears on `PATH`, `osmprj --help` succeeds, and `osmprj themes list` lists themes correctly
- [ ] 4.3 Run `scripts/install.sh` on macOS `osx-arm64` and confirm the same
- [ ] 4.4 Re-run `scripts/install.sh` on a machine where it was already run and confirm no duplicate lines are added to the rc file
- [ ] 4.5 Create a test GitHub release (pre-release tag) and confirm the workflow runs to completion and all three release assets appear (`osmprj-linux-64-installer.sh`, `osmprj-osx-arm64-installer.sh`, `install.sh`)
