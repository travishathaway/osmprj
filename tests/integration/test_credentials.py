"""Integration tests for credential resolution (OSMPRJ_DATABASE_URL)."""


def test_env_var_overrides_inline_database_url(run, monkeypatch, tmp_path):
    """OSMPRJ_DATABASE_URL env var overrides database_url in osmprj.toml."""
    # Init with an inline URL that points nowhere.
    run("init", "--db", "postgres://inline:inlinepass@127.0.0.1:1/nodb", cwd=tmp_path)

    # Set the env var to a different (also unreachable) URL.
    monkeypatch.setenv("OSMPRJ_DATABASE_URL", "postgres://env:envpass@127.0.0.1:1/nodb")

    result = run("status", cwd=tmp_path)
    # Status always exits 0; it should attempt to connect to the env var URL,
    # not the inline one.  Both are unreachable so "connection failed" appears,
    # but the displayed URL must be the env var value — with password masked.
    assert result.returncode == 0
    assert "env:****" in result.stdout
    assert "envpass" not in result.stdout
    assert "inline:inlinepass" not in result.stdout


def test_env_var_used_when_no_inline_url(run, monkeypatch, tmp_path):
    """OSMPRJ_DATABASE_URL is used even when osmprj.toml has no database_url."""
    run("init", cwd=tmp_path)

    monkeypatch.setenv("OSMPRJ_DATABASE_URL", "postgres://env:envpass@127.0.0.1:1/nodb")

    result = run("status", cwd=tmp_path)
    assert result.returncode == 0
    # The env var URL should be shown with password masked (connection will fail).
    assert "env:****" in result.stdout
    assert "envpass" not in result.stdout
    assert "not configured" not in result.stdout
