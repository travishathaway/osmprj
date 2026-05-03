"""
Test suite for the `osmprj add` command.

All of these tests do not require a database connection. to avoid making http
calls, we also use a cache copy of the Geofabrik index.
"""

try:
    import tomllib
except ImportError:
    import tomli as tomllib


def test_fails_without_project(run):
    result = run("add", "--path", "/data/region.pbf", "--name", "myregion")
    assert result.returncode != 0


def test_error_mentions_project_not_found(run):
    result = run("add", "--path", "/data/region.pbf", "--name", "myregion")
    assert "osmprj.toml not found" in result.stderr


def test_add_local_file_succeeds(run, project):
    result = run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    assert result.returncode == 0


def test_add_local_file_prints_success(run, project):
    result = run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    assert "Added [sources.myregion] to osmprj.toml" in result.stdout


def test_add_local_file_writes_source(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert "myregion" in config["sources"]


def test_add_local_file_writes_path(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["myregion"]["path"] == "/data/region.pbf"


def test_default_schema_matches_name(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["myregion"]["schema"] == "myregion"


def test_dash_in_name_normalizes_schema(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "my-region", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["my-region"]["schema"] == "my_region"


def test_explicit_schema_overrides_default(run, project):
    run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--schema",
        "custom_schema",
        cwd=project,
    )
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["myregion"]["schema"] == "custom_schema"


def test_theme_written_when_provided(run, project):
    run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--theme",
        "shortbread",
        cwd=project,
    )
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["myregion"]["theme"] == "shortbread"


def test_no_theme_key_when_omitted(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert "theme" not in config["sources"]["myregion"]


def test_multiple_sources_coexist(run, project):
    run("add", "--path", "/data/a.pbf", "--name", "source-a", cwd=project)
    run("add", "--path", "/data/b.pbf", "--name", "source-b", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert "source-a" in config["sources"]
    assert "source-b" in config["sources"]


def test_duplicate_source_fails(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    result = run("add", "--path", "/data/other.pbf", "--name", "myregion", cwd=project)
    assert result.returncode != 0


def test_duplicate_source_error_mentions_name(run, project):
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=project)
    result = run("add", "--path", "/data/other.pbf", "--name", "myregion", cwd=project)
    assert "myregion" in result.stderr
    assert "already exists" in result.stderr


def test_path_without_name_fails(run, project):
    result = run("add", "--path", "/data/region.pbf", cwd=project)
    assert result.returncode != 0


def test_path_without_name_error_message(run, project):
    result = run("add", "--path", "/data/region.pbf", cwd=project)
    assert "--name is required" in result.stderr


def test_add_geofabrik_id_succeeds(run, project):
    result = run("add", "albania", cwd=project)
    assert result.returncode == 0


def test_add_geofabrik_id_prints_success(run, project):
    result = run("add", "albania", cwd=project)
    assert "Added [sources.albania] to osmprj.toml" in result.stdout


def test_add_geofabrik_id_sets_schema(run, project):
    run("add", "albania", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["albania"]["schema"] == "albania"


def test_add_geofabrik_id_has_no_path(run, project):
    run("add", "albania", cwd=project)
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert "path" not in config["sources"]["albania"]


def test_add_geofabrik_slash_id_normalizes_schema(run, project):
    result = run("add", "us/alabama", cwd=project)
    assert result.returncode == 0
    config = tomllib.loads((project / "osmprj.toml").read_text())
    assert config["sources"]["us/alabama"]["schema"] == "us_alabama"


def test_add_unknown_geofabrik_id_fails(run, project):
    result = run("add", "not-a-real-region-xyzzy", cwd=project)
    assert result.returncode != 0


def test_add_unknown_geofabrik_id_error_mentions_id(run, project):
    result = run("add", "not-a-real-region-xyzzy", cwd=project)
    assert "not-a-real-region-xyzzy" in result.stderr


def test_add_nonexistent_plugin_theme_fails(run, project):
    """Osmprj add with an unrecognised theme name must exit non-zero."""
    result = run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--theme",
        "nonexistent-theme-xyzzy",
        cwd=project,
    )
    assert result.returncode != 0


def test_add_nonexistent_plugin_theme_error_mentions_name(run, project):
    """The error message must mention the theme name."""
    result = run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--theme",
        "nonexistent-theme-xyzzy",
        cwd=project,
    )
    assert "nonexistent-theme-xyzzy" in result.stderr


def test_add_nonexistent_plugin_theme_error_mentions_searched_path(run, project):
    """The error message must mention at least one searched path."""
    result = run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--theme",
        "nonexistent-theme-xyzzy",
        cwd=project,
    )
    # The error should list at least one filesystem path that was searched.
    combined = result.stdout + result.stderr
    assert "share" in combined or "osmprj" in combined


def test_add_builtin_theme_succeeds(run, project):
    """A built-in themepark theme name must be accepted without error."""
    result = run(
        "add",
        "--path",
        "/data/region.pbf",
        "--name",
        "myregion",
        "--theme",
        "shortbread",
        cwd=project,
    )
    assert result.returncode == 0
