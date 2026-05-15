## Context

`osmprj` is a Rust CLI tool that wraps `osm2pgsql`, `osm2pgsql-replication`, and `pyosmium`. It ships as a conda package via the `gis-forge` channel. The repo uses `pixi` for environment management, `rattler-build` for conda packaging, and GitHub Actions for CI. Supported platforms are `linux-64` and `osx-arm64`.

`pixi-pack` is a tool by Quantco that packs a pixi/conda environment (using `pixi.lock`) into a self-extracting archive. With `--create-executable` it produces an `environment.sh` (Unix) or `environment.ps1` (Windows) that embeds both `pixi-unpack` and all conda packages. The self-extracting script passes `"$@"` through to the embedded `pixi-unpack` binary, which supports `--output-directory <DIR>` and `--env-name <NAME>`.

## Goals / Non-Goals

**Goals:**
- One-liner install experience: `curl https://github.com/travishathaway/osmprj/releases/latest/download/install.sh | bash`
- `osmprj` available on `PATH` after install, no manual activation required.
- `OSMPRJ_THEME_PATH` correctly set so themes are found.
- Triggered automatically on GitHub release publish.
- Platform support: `linux-64`, `osx-arm64`.

**Non-Goals:**
- Windows support (deferred).
- `osx-64` (Intel Mac) support (deferred).
- Uninstall script.
- Upgrading an existing installation (user re-runs the installer).

## Decisions

### 1. pixi.toml: new `pack` feature and environment

Add a minimal `pack` feature containing only what the release CI needs — `rattler-build` and `pixi-pack`. Keep it `no-default-feature = true` so it is lean and isolated from the runtime environment being packed.

```toml
[feature.pack.dependencies]
rattler-build = "*"
pixi-pack = "*"

[environments]
pack = { features = ["pack"], no-default-feature = true }
```

Note: `rattler-build` is also present in `[feature.dev.dependencies]`. The duplication is intentional — `pack` must be self-contained without pulling in all dev tooling.

### 2. Release CI: two-job structure

```
release.yml
═══════════════════════════════════════════════════════

  trigger: on.release.types: [published]

  jobs:
  ┌─────────────────────────────────────────────────┐
  │ build-installer (matrix)                        │
  │   matrix:                                       │
  │     platform: linux-64  / runner: ubuntu-latest │
  │     platform: osx-arm64 / runner: macos-14      │
  │                                                 │
  │   steps:                                        │
  │   1. actions/checkout@v4                        │
  │   2. prefix-dev/setup-pixi@v0.8.5               │
  │        environments: rust pack                  │
  │   3. build binary                               │
  │        pixi run -e rust release                 │
  │        → target/release/osmprj                  │
  │   4. build conda package                        │
  │        pixi run -e pack rattler-build build     │
  │          -r recipe/recipe.yaml                  │
  │        → output/<subdir>/osmprj-*.conda         │
  │   5. pack environment                           │
  │        pixi run -e pack pixi-pack pack          │
  │          --environment default                  │
  │          --platform ${{ matrix.platform }}      │
  │          --inject output/**/*.conda             │
  │          --create-executable                    │
  │          pixi.toml                              │
  │        → environment.sh                         │
  │   6. rename artifact                            │
  │        mv environment.sh                        │
  │           osmprj-${{ matrix.platform }}         │
  │           -installer.sh                         │
  │   7. actions/upload-artifact@v4                 │
  │        name: installer-${{ matrix.platform }}   │
  │        path: osmprj-*-installer.sh              │
  └─────────────────────────────────────────────────┘
              │
              ▼
  ┌─────────────────────────────────────────────────┐
  │ publish-release                                 │
  │   needs: build-installer                        │
  │   runs-on: ubuntu-latest                        │
  │                                                 │
  │   steps:                                        │
  │   1. actions/checkout@v4                        │
  │   2. actions/download-artifact@v4               │
  │        path: installers/                        │
  │        pattern: installer-*                     │
  │        merge-multiple: true                     │
  │   3. gh release upload ${{ github.ref_name }}   │
  │        installers/osmprj-linux-64-installer.sh  │
  │        installers/osmprj-osx-arm64-installer.sh │
  │        scripts/install.sh                       │
  └─────────────────────────────────────────────────┘
```

### 3. rattler-build invocation

The recipe at `recipe/recipe.yaml` uses `source.path: ../` (relative to the recipe). It builds `osmprj` via `cargo auditable install`. This works in CI because the `rust` environment is set up first (step 3 above builds the binary), but `rattler-build` runs its own isolated build environment — it will download and use its own Rust toolchain.

The `rattler-build build` command is run from the repo root:
```bash
rattler-build build -r recipe/recipe.yaml --output-dir output/
```

Output location: `output/<platform>/osmprj-<version>-<hash>_<build>.conda`

### 4. pixi-pack invocation

```bash
pixi-pack pack \
  --environment default \
  --platform <platform> \
  --inject output/**/*.conda \
  --create-executable \
  pixi.toml
```

This produces `environment.sh` in the current directory. The `--inject` flag folds the locally-built `osmprj` conda package into the pack alongside all default-environment dependencies. pixi-pack verifies injected package compatibility before creating the archive.

### 5. install.sh: platform detection and install flow

```
scripts/install.sh
══════════════════════════════════════════════════════

  INSTALL_DIR="${OSMPRJ_INSTALL_DIR:-$HOME/.local/osmprj}"
  RELEASE_BASE="https://github.com/travishathaway/osmprj/releases/latest/download"

  1. detect platform:
       uname -s → Linux  → "linux"
       uname -s → Darwin → "darwin"
       uname -m → x86_64 → linux-64
       uname -m → arm64  → osx-arm64
       anything else     → unsupported, print message, exit 1

  2. download installer:
       curl -fsSL "$RELEASE_BASE/osmprj-${PLATFORM}-installer.sh"
         -o /tmp/osmprj-installer.sh
       chmod +x /tmp/osmprj-installer.sh

  3. run installer:
       /tmp/osmprj-installer.sh \
         --output-directory "$INSTALL_DIR"
       → unpacks to $INSTALL_DIR/env/
       → writes $INSTALL_DIR/activate.sh

  4. detect shell rc file:
       $SHELL contains "zsh"  → ~/.zshrc
       $SHELL contains "bash" → ~/.bashrc
       fallback               → ~/.profile

  5. append to rc file (idempotent — skip if already present):
       # osmprj
       export PATH="$INSTALL_DIR/env/bin:$PATH"
       export OSMPRJ_THEME_PATH="$INSTALL_DIR/env/share/osmprj/themes/"

  6. cleanup:
       rm /tmp/osmprj-installer.sh

  7. print success:
       osmprj has been installed to $INSTALL_DIR
       Restart your shell or run:
         source ~/.bashrc   (or ~/.zshrc)
```

### 6. OSMPRJ_THEME_PATH override rationale

The conda package's `etc/conda/env_vars.d/osmprj.json` sets `OSMPRJ_THEME_PATH` to the build-time prefix (a path on the CI runner). This is wrong on the user's machine. The install script overrides it by exporting the correct path in the shell rc file. Since the conda activation hook only runs when `source activate.sh` is called, and normal use never calls that, the rc-file export is the sole source of truth.

## Risks / Trade-offs

- **`rattler-build` self-contained build**: rattler-build downloads its own Rust toolchain and builds `osmprj` fresh. This means the conda package binary differs from the `target/release/osmprj` built in step 3. Step 3 is actually only needed to satisfy the existing CI test jobs — for the release pipeline it is redundant. Consider whether step 3 can be removed (it can, once release.yml is standalone).
- **`output/**/*.conda` glob**: The exact path of the rattler-build output depends on the version string and build hash. Using a glob (`output/**/*.conda`) is robust but assumes only one `.conda` file is produced. If rattler-build produces multiple, pixi-pack will attempt to inject all of them — which may cause a solver error. The glob should target the platform subdir: `output/linux-64/*.conda` or `output/osx-arm64/*.conda`.
- **`pixi-pack` cross-platform pack**: pixi-pack can pack for any platform from any runner (it just downloads the right conda packages). However, the `rattler-build` step must run natively on each platform to produce the correct platform binary. The matrix approach is therefore required.
- **Idempotency of rc file edits**: The install script must check for an existing `# osmprj` block before appending to avoid duplicate entries on re-install.
- **`macos-14` runner for `osx-arm64`**: GitHub's `macos-14` runner is Apple Silicon (arm64). `macos-latest` may vary — pin to `macos-14` for reliability.
