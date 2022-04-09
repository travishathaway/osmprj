from pathlib import Path
from urllib.parse import urljoin, urlparse

import click
import requests
from rich.progress import track


def is_valid_resource(url: str) -> bool:
    """
    Perform a HEAD request and see if there's anything there.
    """
    try:
        resp = requests.head(url, timeout=5)
    except requests.exceptions.ReadTimeout:
        raise click.ClickException('Request timed out. Unable to verify GeoFabrik resource (server could be down 😭).')
    return 200 <= resp.status_code < 400


def download_url(url: str, dest: Path, show_progress: bool = True) -> None:
    r = requests.get(url, stream=True)
    chunk_size = 1024
    size = int(r.headers.get('Content-Length', 0)) / chunk_size

    with dest.open('wb') as f:
        resp_iter = r.iter_content(chunk_size=chunk_size)

        if show_progress:
            resp_iter = track(resp_iter, description='Downloading...', total=round(size))

        for chunk in resp_iter:
            if chunk:
                f.write(chunk)
                f.flush()


def build_url_from_desc(desc: str) -> str:
    """
    Builds a new URL for a GeoFabrik download.
    >>> build_url_from_desc('europe/germany')
    'https://download.geofabrik.de/europe/germany-latest.osm.pbf'
    """
    base_url = 'https://download.geofabrik.de'
    url = urljoin(base_url, f'{desc}-latest.osm.pbf')

    return url


def urlbasename(url: str) -> str:
    """
    Return the basename of a URL
    >>> urlbasename('https://example.com/path/to/a/resource.txt')
    'resource.txt'
    >>> urlbasename('https://example.com')
    ''
    """
    return Path(urlpath(url)).name


def urlpath(url: str) -> str:
    """Return the path of a URL as Path object"""
    return urlparse(url).path
