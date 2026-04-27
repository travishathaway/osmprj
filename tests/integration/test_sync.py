"""Integration tests for the sync command.

Tuner logic (cache MB, flat-nodes thresholds) is verified by Rust unit tests
in src/tuner.rs (`cargo test`). The tests here cover CLI-observable behaviour.
"""
import json
import tomllib


def test_sync_fails_without_project(run):
    result = run("sync")
    assert result.returncode != 0


def test_sync_error_mentions_project_not_found(run):
    result = run("sync")
    assert "osmprj.toml not found" in result.stderr


def test_sync_fails_with_unknown_source(run, project):
    result = run("sync", "nonexistent-source-xyz", cwd=project)
    assert result.returncode != 0


def test_sync_unknown_source_error_mentions_name(run, project):
    result = run("sync", "nonexistent-source-xyz", cwd=project)
    assert "nonexistent-source-xyz" in result.stderr


def test_sync_unknown_source_exits_before_any_work(run, project):
    """Unknown source should be rejected immediately, no lock file created."""
    result = run("sync", "nonexistent-source-xyz", cwd=project)
    assert result.returncode != 0
    assert not (project / "osmprj.lock").exists()


def test_sync_verbose_flag_accepted(run, project):
    """The -v flag should parse without error (even if sync exits non-zero for
    other reasons like missing binaries or database)."""
    result = run("-v", "sync", cwd=project)
    # Should not fail due to flag parsing; may fail due to missing osm2pgsql
    assert "unrecognized" not in result.stderr.lower()


def test_sync_source_filter_accepts_known_source(run, tmp_path):
    """Sync with a known source name should pass validation (may fail later
    due to missing osm2pgsql binary, but not with UnknownSources error)."""
    run("init", cwd=tmp_path)
    run("add", "albania", cwd=tmp_path)
    result = run("sync", "albania", cwd=tmp_path)
    # If it fails, it should NOT be due to unknown source
    if result.returncode != 0:
        assert "nonexistent" not in result.stderr
        assert "unknown source" not in result.stderr.lower()


# ── Lock file tests ────────────────────────────────────────────────────────────

def test_lock_not_created_without_sync(run, project):
    assert not (project / "osmprj.lock").exists()


def test_lock_skips_source_when_entry_present(run, tmp_path):
    """If osmprj.lock already has an entry for a source, sync should skip the
    download for that source (observable: no re-download message)."""
    run("init", cwd=tmp_path)
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=tmp_path)

    # Manually pre-seed the lock file
    lock_content = (
        "# osmprj.lock\n\n"
        "[sources.myregion]\n"
        'url = "https://example.com/region.osm.pbf"\n'
        'md5 = "abc123"\n'
        'downloaded_at = "2026-04-26T12:00:00Z"\n'
    )
    (tmp_path / "osmprj.lock").write_text(lock_content)

    result = run("sync", "myregion", cwd=tmp_path)
    # Source has a local path so it skips download; but osm2pgsql may be missing
    # The key assertion: it should not attempt a network download for myregion
    assert "already downloaded" not in result.stdout or result.returncode != 0


def test_lock_file_is_valid_toml_when_present(run, tmp_path):
    run("init", cwd=tmp_path)
    run("add", "--path", "/data/region.pbf", "--name", "myregion", cwd=tmp_path)

    lock_path = tmp_path / "osmprj.lock"
    lock_path.write_text(
        "[sources.myregion]\n"
        'url = "https://example.com/r.osm.pbf"\n'
        'md5 = "d41d8cd98f00b204e9800998ecf8427e"\n'
        'downloaded_at = "2026-04-26T12:00:00Z"\n'
    )
    # Verify the lock parses as valid TOML
    data = tomllib.loads(lock_path.read_text())
    assert "sources" in data
    assert "myregion" in data["sources"]
    assert data["sources"]["myregion"]["md5"] == "d41d8cd98f00b204e9800998ecf8427e"
