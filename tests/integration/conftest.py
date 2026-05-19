"""
Holds all fixtures for our test suite.

These are mostly used for making subshell calls and setting up and
tearing down databases.
"""

import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import threading
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import psycopg
import pytest
from pg_helper.postgres import PgDataManager, Platform, PostgresManager
from xprocess import ProcessStarter

# Make _platform.py importable from test modules in this directory.
sys.path.insert(0, str(Path(__file__).parent))

from _platform import platform_cache_env, platform_cache_subdir

BINARY = Path(__file__).parents[2] / "target" / "release" / "osmprj"
DATA_DIR = Path(__file__).parents[1] / "data"
PG_TEST_PORT = 65112
PG_TEST_PASSWORD = "osmprj_test"  # noqa: S105
REPO_ROOT = Path(__file__).parents[2]
TESTS_THEMES_DIR = Path(__file__).parents[1] / "themes"
PBF_FIXTURE_DIR = Path(
    os.environ.get("OSMPRJ_TEST_FIXTURE_DIR", REPO_ROOT / "tests" / "fixtures" / "pbf")
)


def pytest_configure(config):
    (Path(__file__).parents[2] / "reports").mkdir(exist_ok=True)


@pytest.fixture(scope="session")
def binary():
    if not BINARY.exists():
        pytest.skip(f"Binary not found at {BINARY}; run 'cargo build --release' first.")
    return BINARY


@pytest.fixture(scope="session")
def run_cmd(binary):
    """Session-scoped helper that runs the osmprj binary and returns CompletedProcess.

    Pass ``check=False`` to suppress the assertion on returncode (e.g. when you
    want to capture a result that may fail and assert on it yourself).
    """

    def _run_cmd(*args, cwd, check=True):
        result = subprocess.run(
            [str(binary), *args], cwd=cwd, capture_output=True, text=True, check=False
        )
        if check:
            assert result.returncode == 0, (
                f"osmprj {' '.join(args)} failed (rc={result.returncode}):\n{result.stderr}"
            )
        return result

    return _run_cmd


@pytest.fixture(scope="session")
def pg_e2e(tmp_path_factory):
    """Ephemeral PostgreSQL cluster on port 65112 with scram-sha-256 auth, managed by pg-helper."""
    tmp_data_dir = tmp_path_factory.mktemp("pg_e2e") / "pgdata"
    mp = pytest.MonkeyPatch()

    # Create .pgpass file (mode 0600 required by libpq)
    pgpass_path = tmp_path_factory.mktemp("pgpass") / ".pgpass"
    pgpass_path.write_text(f"localhost:{PG_TEST_PORT}:*:postgres:{PG_TEST_PASSWORD}\n")
    pgpass_path.chmod(0o600)

    # Expose .pgpass to the entire test session (inherited by all subprocesses)
    mp.setenv("PGPASSFILE", str(pgpass_path))

    # Start with trust auth so we can set the password via SQL
    subprocess.run(
        ["pg-helper", "--data-dir", tmp_data_dir, "--port", str(PG_TEST_PORT), "start"], check=True
    )

    conn_str = f"postgresql://postgres:{PG_TEST_PASSWORD}@localhost:{PG_TEST_PORT}/postgres"

    try:
        trust_conn_str = f"postgresql://postgres@localhost:{PG_TEST_PORT}/postgres"
        with psycopg.connect(trust_conn_str, autocommit=True) as conn:
            conn.execute(f"ALTER USER postgres PASSWORD '{PG_TEST_PASSWORD}'")
            conn.execute("CREATE EXTENSION IF NOT EXISTS hstore")

        # Switch pg_hba.conf from trust to scram-sha-256 and reload
        data_mgr = PgDataManager(tmp_data_dir)
        pg_mgr = PostgresManager(data_mgr, PG_TEST_PORT)
        pg_mgr.configure_password_auth()

        pg_ctl = Platform.find_pg_command("pg_ctl")
        subprocess.run([pg_ctl, "reload", "-D", str(tmp_data_dir)], check=True)

        # Verify password auth works before yielding
        with psycopg.connect(conn_str) as conn:
            conn.execute("SELECT 1")

        yield conn_str

    finally:
        mp.undo()
        subprocess.run(
            ["pg-helper", "--data-dir", tmp_data_dir, "--port", str(PG_TEST_PORT), "stop"],
            check=True,
        )
        subprocess.run(
            [
                "pg-helper",
                "--data-dir",
                tmp_data_dir,
                "--port",
                str(PG_TEST_PORT),
                "destroy",
                "--force",
            ],
            check=True,
        )


@pytest.fixture(scope="session")
def geofabrik_server(xprocess):
    """Mock Geofabrik server."""
    port = "58585"

    class Server(ProcessStarter):
        pattern = f"Running on http://127.0.0.1:{port}"
        args = ("gms", "serve", "--port", port)

    _ = xprocess.ensure("server", Server)

    yield "http://localhost:58585"

    xprocess.getinfo("server").terminate()


@pytest.fixture(scope="session")
def geofabrik_cache_dir(geofabrik_server: str, tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Session-scoped fake cache root pre-populated with the test Geofabrik index.

    The internal layout mirrors what the ``dirs`` crate resolves on each platform:
      Linux/Windows : <root>/osmprj/geofabrik-index-v1.json
      macOS         : <root>/Library/Caches/osmprj/geofabrik-index-v1.json

    The returned path is the root that should be assigned to the platform env var
    (XDG_CACHE_HOME on Linux, HOME on macOS, LOCALAPPDATA on Windows).
    """
    root = tmp_path_factory.mktemp("geofabrik_cache")
    cache_subdir = root / "Library" / "Caches" if platform.system() == "Darwin" else root
    osmprj_dir = cache_subdir / "osmprj"
    osmprj_dir.mkdir(parents=True)
    shutil.copy(DATA_DIR / "geofabrik-index-v1.json", osmprj_dir / "geofabrik-index-v1.json")

    index = Path(osmprj_dir / "geofabrik-index-v1.json")
    new_text = index.read_text().replace("https://download.geofabrik.de", geofabrik_server)
    index.write_text(new_text)

    return root


@pytest.fixture
def run(binary, tmp_path, geofabrik_cache_dir, monkeypatch):
    system = platform.system()
    if system == "Darwin":
        monkeypatch.setenv("HOME", str(geofabrik_cache_dir))
    elif system == "Windows":
        monkeypatch.setenv("LOCALAPPDATA", str(geofabrik_cache_dir))
    else:
        monkeypatch.setenv("XDG_CACHE_HOME", str(geofabrik_cache_dir))

    def _run(*args, cwd=None):
        return subprocess.run(
            [str(binary), *args], cwd=cwd or tmp_path, capture_output=True, text=True, check=False
        )

    return _run


@pytest.fixture
def project(run, tmp_path):
    """Temporary directory with osmprj already initialized."""
    result = run("init")
    assert result.returncode == 0
    return tmp_path


# platform helpers are imported from _platform.py


# ─── PBF fixture cache ────────────────────────────────────────────────────────

_MONACO_REGION = "europe/monaco-latest"
_LIECHTENSTEIN_REGION = "europe/liechtenstein-latest"
_GEOFABRIK_BASE = "https://download.geofabrik.de"
REGIONS = [_MONACO_REGION, _LIECHTENSTEIN_REGION]


@pytest.fixture(scope="session")
def pbf_fixture_cache():
    """Download the `REGIONS` pbf files once and cache it locally.

    Default location: <repo_root>/tests/fixtures/pbf/ (gitignored).
    Override with OSMPRJ_TEST_FIXTURE_DIR env var.

    Also computes and saves a .md5 sidecar in Geofabrik format.
    Returns the cache root Path.
    """
    for region in REGIONS:
        pbf_path = PBF_FIXTURE_DIR / f"{region}.osm.pbf"
        md5_path = PBF_FIXTURE_DIR / f"{region}.osm.pbf.md5"

        pbf_path.parent.mkdir(parents=True, exist_ok=True)

        if not pbf_path.exists():
            url = f"{_GEOFABRIK_BASE}/{region}.osm.pbf"
            urllib.request.urlretrieve(url, pbf_path)  # noqa: S310

        if not md5_path.exists():
            digest = hashlib.md5(pbf_path.read_bytes(), usedforsecurity=False).hexdigest()
            md5_path.write_text(f"{digest}  {pbf_path.name}\n")

    return PBF_FIXTURE_DIR


# ─── TestOsmServer ────────────────────────────────────────────────────────────


class _Injection:
    """Holds a single injected response behavior and its remaining count."""

    __slots__ = ("bytes_to_send", "kind", "remaining", "status")

    def __init__(self, kind, *, status=None, bytes_to_send=None, times=1):
        self.kind = kind  # "status" | "bad_md5" | "partial"
        self.status = status
        self.bytes_to_send = bytes_to_send
        self.remaining = times


class _OsmHttpServer(ThreadingHTTPServer):
    """ThreadingHTTPServer that carries a reference to its owning TestOsmServer."""

    def __init__(self, addr: tuple, handler_class: type, test_server: "TestOsmServer") -> None:
        super().__init__(addr, handler_class)
        self.test_server = test_server


class _OsmHandler(BaseHTTPRequestHandler):
    """HTTP request handler that serves PBF fixtures with optional error injection."""

    def log_message(self, *_: object) -> None:
        pass  # suppress request logging in test output

    def _ts(self) -> "TestOsmServer":
        return self.server.test_server  # type: ignore[attr-defined]

    def _resolve(self) -> Path:
        return self._ts().fixture_dir / self.path.lstrip("/")

    def _consume_injection(self) -> "_Injection | None":
        return self._ts().consume_injection(self.path)

    def do_HEAD(self) -> None:
        """Respond to HEAD with 200+Content-Length or 404."""
        file_path = self._resolve()
        if file_path.exists():
            self.send_response(200)
            self.send_header("Content-Length", str(file_path.stat().st_size))
            self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()

    def do_GET(self) -> None:
        """Respond to GET, honouring any active injection."""
        inj = self._consume_injection()

        if inj is not None:
            if inj.kind == "status":
                self.send_response(inj.status)
                self.end_headers()
                return

            if inj.kind == "bad_md5":
                body = b"00000000000000000000000000000000  dummy.osm.pbf\n"
                self.send_response(200)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return

            if inj.kind == "partial":
                file_path = self._resolve()
                if not file_path.exists():
                    self.send_response(404)
                    self.end_headers()
                    return
                real_size = file_path.stat().st_size
                self.send_response(200)
                self.send_header("Content-Length", str(real_size))
                self.end_headers()
                data = file_path.read_bytes()
                self.wfile.write(data[: inj.bytes_to_send])
                self.wfile.flush()
                # Force close so client sees a truncated body
                self.close_connection = True
                return

        file_path = self._resolve()
        if file_path.exists():
            data = file_path.read_bytes()
            self.send_response(200)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        else:
            self.send_response(404)
            self.end_headers()


class TestOsmServer:
    """Local HTTP server that serves cached PBF fixtures with optional error injection."""

    def __init__(self, fixture_dir: Path, port: int = 0) -> None:
        self._fixture_dir = fixture_dir
        self._lock = threading.Lock()
        self._errors: dict = {}
        self._server = _OsmHttpServer(("127.0.0.1", port), _OsmHandler, self)
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    @property
    def fixture_dir(self) -> Path:
        """Return the fixture directory served by this server."""
        return self._fixture_dir

    def consume_injection(self, path: str) -> "_Injection | None":
        """Consume and return the active injection for *path*, or None."""
        with self._lock:
            inj = self._errors.get(path)
            if inj is None:
                return None
            inj.remaining -= 1
            if inj.remaining <= 0:
                del self._errors[path]
            return inj

    @property
    def base_url(self) -> str:
        """Return the server base URL as http://host:port."""
        host, port = self._server.server_address
        return f"http://{host}:{port}"

    def start(self) -> None:
        """Start the server thread."""
        self._thread.start()

    def stop(self) -> None:
        """Shut down the server."""
        self._server.shutdown()

    def inject_error(self, path: str, *, status: int, times: int = 1) -> None:
        """Inject an HTTP error status for *path* on the next *times* requests."""
        with self._lock:
            self._errors[path] = _Injection("status", status=status, times=times)

    def inject_bad_md5(self, path: str, *, times: int = 1) -> None:
        """Inject an incorrect MD5 body for *path* on the next *times* requests."""
        with self._lock:
            self._errors[path] = _Injection("bad_md5", times=times)

    def inject_partial(self, path: str, *, bytes_to_send: int, times: int = 1) -> None:
        """Inject a truncated response for *path*, closing after *bytes_to_send* bytes."""
        with self._lock:
            self._errors[path] = _Injection("partial", bytes_to_send=bytes_to_send, times=times)

    def reset_errors(self) -> None:
        """Clear all pending error injections."""
        with self._lock:
            self._errors.clear()


# ─── server fixtures ──────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def download_server(pbf_fixture_cache):
    """Session-scoped local HTTP server serving PBF fixtures with error injection support."""
    server = TestOsmServer(pbf_fixture_cache)
    server.start()
    yield server
    server.stop()


@pytest.fixture(scope="session")
def geofabrik_cache_with_server(tmp_path_factory, download_server):
    """Session-scoped cache root with the Geofabrik index rewritten to localhost URLs.

    All properties.urls.pbf values have https://download.geofabrik.de replaced with
    the test server base_url. The index is written in the platform-correct layout so
    the binary reads it via the platform cache env var. Setting that env var also
    redirects effective_data_dir() into the same tmp tree.
    """
    root = tmp_path_factory.mktemp("geofabrik_cache_server")
    osmprj_dir = platform_cache_subdir(root) / "osmprj"
    osmprj_dir.mkdir(parents=True, exist_ok=True)

    raw = json.loads((DATA_DIR / "geofabrik-index-v1.json").read_text())
    for feature in raw.get("features", []):
        urls = feature.get("properties", {}).get("urls", {})
        if "pbf" in urls:
            urls["pbf"] = urls["pbf"].replace(_GEOFABRIK_BASE, download_server.base_url)

    (osmprj_dir / "geofabrik-index-v1.json").write_text(json.dumps(raw))
    return root


@pytest.fixture(scope="session")
def run_cmd_with_server(binary, geofabrik_cache_with_server):
    """Like run_cmd but injects the platform cache env var pointing at the test server index."""
    env_key, env_val = platform_cache_env(geofabrik_cache_with_server)

    def _run(*args, cwd, check=True):
        env = {**os.environ.copy(), env_key: env_val}
        result = subprocess.run(
            [str(binary), *args], cwd=cwd, capture_output=True, text=True, check=False, env=env
        )
        if check:
            assert result.returncode == 0, (
                f"osmprj {' '.join(args)} failed (rc={result.returncode}):\n{result.stderr}"
            )
        return result

    return _run


@pytest.fixture
def reset_server_errors(download_server):
    """Reset all injected server errors after the test completes."""
    yield
    download_server.reset_errors()


@pytest.fixture(autouse=True)
def tests_themes_dir(monkeypatch: pytest.MonkeyPatch) -> Path:
    """Path to tests/themes/, containing test-only theme packages.

    We automatically set this for the entire test suite so we can test themes
    that aren't included in our default set (e.g. themes that are broken or cause
    various errors).
    """
    existing = os.environ.get("OSMPRJ_THEME_PATH", "")
    extended = str(TESTS_THEMES_DIR) + (os.pathsep + existing if existing else "")

    monkeypatch.setenv("OSMPRJ_THEME_PATH", extended)

    return TESTS_THEMES_DIR
