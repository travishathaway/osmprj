"""Tests for commands that are not yet implemented (sync, remove, destroy)."""


def test_sync_exits_zero(run):
    result = run("sync")
    assert result.returncode == 0


def test_sync_prints_not_implemented(run):
    result = run("sync")
    assert "not yet implemented" in result.stdout


def test_remove_exits_zero(run):
    result = run("remove")
    assert result.returncode == 0


def test_remove_prints_not_implemented(run):
    result = run("remove")
    assert "not yet implemented" in result.stdout


def test_destroy_exits_zero(run):
    result = run("destroy")
    assert result.returncode == 0


def test_destroy_prints_not_implemented(run):
    result = run("destroy")
    assert "not yet implemented" in result.stdout
