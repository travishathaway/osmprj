## Why

`osmprj` currently requires users to have `pixi` or `conda` installed before they can install the tool. This is a significant barrier for GIS practitioners and sysadmins who just want to run `curl https://example.com/install.sh | bash` and get a working `osmprj` on their PATH — the same one-liner experience offered by tools like `rustup`, `Homebrew`, and `uv`.

The tool also carries substantial native dependencies (`osm2pgsql`, `pyosmium`, `psycopg`, themepark files) that are non-trivial to install manually. A self-contained installer that bundles everything removes all of that friction.

## What Changes

- Add a `scripts/install.sh` to the repository: a thin platform-detecting shell script that users can `curl | bash`.
- Add a GitHub Actions release workflow (`.github/workflows/release.yml`) that triggers when a GitHub release is published.
- The release workflow builds the `osmprj` conda package with `rattler-build`, then uses `pixi-pack --create-executable` to produce a self-extracting installer for each supported platform.
- Add `pixi-pack` and `rattler-build` to `pixi.toml` under a new `pack` feature/environment.
- The self-extracting installers and `install.sh` are uploaded as release assets.

## Capabilities

### New Capabilities

- `curl-installer`: `scripts/install.sh` detects the user's platform, downloads the correct platform-specific self-extracting installer from the GitHub release, runs it to unpack the full environment to `~/.local/osmprj/`, and wires `PATH` and `OSMPRJ_THEME_PATH` into the user's shell rc file.
- `release-pipeline`: A GitHub Actions workflow triggered on `release: published` that builds the conda package, produces per-platform self-extracting installers via `pixi-pack --create-executable`, and uploads them as release assets.
- `pack-environment`: A new `pack` pixi feature/environment in `pixi.toml` containing `pixi-pack` and `rattler-build` for use in CI.

## Testing

- Running `scripts/install.sh` on a clean Linux `linux-64` machine must result in `osmprj` being available on `PATH` with correct `OSMPRJ_THEME_PATH`.
- Running `scripts/install.sh` on a clean macOS `osx-arm64` machine must produce the same result.
- `osmprj --help` and `osmprj themes list` must succeed after install (confirming the binary works and themes are found).
- The GitHub Actions release workflow must complete successfully on a real tag push and produce the expected release assets.

## Impact

- `pixi.toml`: Add `[feature.pack.dependencies]` with `pixi-pack` and `rattler-build`; add `pack` environment.
- `scripts/install.sh`: New installer script (uploaded to releases).
- `.github/workflows/release.yml`: New GitHub Actions workflow.
- No changes to `src/`, `Cargo.toml`, `recipe/`, or existing workflows.
