"""Integration tests for credential resolution (OSMPRJ_DATABASE_URL and database_url_command)."""

import sys


def test_env_var_overrides_inline_database_url(run, monkeypatch, tmp_path):
    """OSMPRJ_DATABASE_URL env var overrides database_url in osmprj.toml."""
    # Init with an inline URL that points nowhere.
    run("init", "--db", "postgres://inline:inlinepass@127.0.0.1:1/nodb", cwd=tmp_path)

    # Set the env var to a different (also unreachable) URL.
    monkeypatch.setenv("OSMPRJ_DATABASE_URL", "postgres://env:envpass@127.0.0.1:1/nodb")

    result = run("status", cwd=tmp_path)
    # Status always exits 0; it should attempt to connect to the env var URL,
    # not the inline one.  Both are unreachable so "connection failed" appears,
    # but the displayed URL must be the env var value.
    assert result.returncode == 0
    assert "env:envpass" in result.stdout
    assert "inline:inlinepass" not in result.stdout


def test_env_var_used_when_no_inline_url(run, monkeypatch, tmp_path):
    """OSMPRJ_DATABASE_URL is used even when osmprj.toml has no database_url."""
    run("init", cwd=tmp_path)

    monkeypatch.setenv("OSMPRJ_DATABASE_URL", "postgres://env:envpass@127.0.0.1:1/nodb")

    result = run("status", cwd=tmp_path)
    assert result.returncode == 0
    # The env var URL should be shown (connection will fail, but URL is displayed).
    assert "env:envpass" in result.stdout
    assert "not configured" not in result.stdout


def test_database_url_command_provides_url(run, tmp_path):
    """database_url_command with a simple echo command is used as the database URL."""
    run("init", cwd=tmp_path)

    # Write a database_url_command that echoes an unreachable URL.
    toml_path = tmp_path / "osmprj.toml"
    original = toml_path.read_text()
    # Use echo; the URL won't connect but status should attempt it.
    if sys.platform == "win32":
        cmd = "echo postgres://cmd:cmdpass@127.0.0.1:1/nodb"
    else:
        cmd = "echo 'postgres://cmd:cmdpass@127.0.0.1:1/nodb'"
    toml_path.write_text(original + f'\ndatabase_url_command = "{cmd}"\n')

    result = run("status", cwd=tmp_path)
    assert result.returncode == 0
    assert "cmd:cmdpass" in result.stdout
    assert "not configured" not in result.stdout


def test_database_url_command_nonzero_exit_fails(run, tmp_path):
    """database_url_command that exits non-zero causes osmprj to exit with an error."""
    run("init", cwd=tmp_path)

    toml_path = tmp_path / "osmprj.toml"
    original = toml_path.read_text()
    if sys.platform == "win32":
        cmd = "exit 1"
    else:
        cmd = "exit 1"
    toml_path.write_text(original + f'\ndatabase_url_command = "{cmd}"\n')

    result = run("status", cwd=tmp_path)
    assert result.returncode != 0
    # Error message should mention the credential command failure.
    assert "credential" in result.stderr.lower() or "command" in result.stderr.lower()
