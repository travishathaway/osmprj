try:
    import tomllib
except ImportError:
    import tomli as tomllib


def test_creates_config_file(run, tmp_path):
    result = run("init")
    assert result.returncode == 0
    assert (tmp_path / "osmprj.toml").exists()


def test_prints_success_message(run):
    result = run("init")
    assert "Created osmprj.toml" in result.stdout


def test_config_has_project_section(run, tmp_path):
    run("init")
    config = tomllib.loads((tmp_path / "osmprj.toml").read_text())
    assert "project" in config


def test_db_url_written_to_config(run, tmp_path):
    run("init", "--db", "postgres://user:pass@localhost/osm")
    config = tomllib.loads((tmp_path / "osmprj.toml").read_text())
    assert config["project"]["database_url"] == "postgres://user:pass@localhost/osm"


def test_no_db_url_leaves_commented_placeholder(run, tmp_path):
    run("init")
    content = (tmp_path / "osmprj.toml").read_text()
    assert "# database_url" in content


def test_fails_if_config_already_exists(run):
    run("init")
    result = run("init")
    assert result.returncode != 0


def test_error_message_mentions_existing_file(run):
    run("init")
    result = run("init")
    assert "osmprj.toml already exists" in result.stderr
