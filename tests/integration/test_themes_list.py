"""Integration tests for `osmprj themes list`."""

import os
import subprocess
from pathlib import Path


def test_themes_list_exits_zero(run, tmp_path):
    """Osmprj themes list must exit 0 even outside a project directory."""
    result = run("themes", "list", cwd=tmp_path)
    assert result.returncode == 0


def test_themes_list_shows_builtin_themes(run, tmp_path):
    """The output must include the known built-in themepark theme names."""
    result = run("themes", "list", cwd=tmp_path)
    combined = result.stdout + result.stderr
    for name in ("shortbread", "basic", "osmcarto"):
        assert name in combined, f"Expected built-in theme '{name}' in output"


def test_themes_list_shows_plugin_theme(run, tmp_path, tmp_path_factory):
    """A theme directory placed on OSMPRJ_THEME_PATH must appear in the listing."""
    # Create a minimal plugin theme.
    themes_root = tmp_path_factory.mktemp("themes_root")
    theme_dir = themes_root / "my-test-theme"
    theme_dir.mkdir()
    (theme_dir / "theme.toml").write_text(
        'name = "my-test-theme"\n'
        'version = "0.1.0"\n'
        'description = "Integration test theme"\n'
        'type = "flex"\n'
        'entry = "my-test-theme.lua"\n'
    )
    (theme_dir / "my-test-theme.lua").write_text("-- stub flex style\n")

    env = {**os.environ, "OSMPRJ_THEME_PATH": str(themes_root)}

    binary = Path(__file__).parents[2] / "target" / "release" / "osmprj"
    result = subprocess.run(
        [str(binary), "themes", "list"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )
    assert result.returncode == 0
    assert "my-test-theme" in result.stdout + result.stderr


def test_themes_list_no_project_required(run, tmp_path):
    """Osmprj themes list must not require osmprj.toml to be present."""
    # tmp_path is a fresh directory with no osmprj.toml
    result = run("themes", "list", cwd=tmp_path)
    assert result.returncode == 0
    assert "osmprj.toml not found" not in result.stderr
