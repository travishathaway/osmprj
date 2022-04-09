import os
from pathlib import Path

import click

from osmprj import http

CACHE_DIR = Path('~/.cache').expanduser().joinpath('osmprj')

if not CACHE_DIR.is_dir():
    CACHE_DIR.mkdir()


def create_cache_dir(cache_dir: Path) -> None:
    """Create a new cache dir if it doesn't exist"""
    if not cache_dir.exists():
        cache_dir.mkdir()


def make_cache_dir_from_url(url: str) -> Path:
    """
    Attempts to make a cache dir from URL path and return a Path object
    """
    path = Path(http.urlpath(url).lstrip('/'))
    new_cache_dir = CACHE_DIR.joinpath(path.parent)

    try:
        os.makedirs(new_cache_dir)
    except FileExistsError:
        pass
    except Exception as exc:
        raise click.ClickException(f'Unable to create cache directory: {exc}')

    return new_cache_dir


def get_cache_file_from_url(url: str) -> Path:
    """
    Ensures a location for our file in our cache and returns a
    usable file path to write to.
    """
    cache_dir = make_cache_dir_from_url(url)
    filename = http.urlbasename(url)
    new_cache_file = cache_dir.joinpath(filename)

    return new_cache_file
