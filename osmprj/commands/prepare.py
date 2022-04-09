import shutil

import click

from osmprj import cache
from osmprj import http
from osmprj import osm
from osmprj import validators


@click.command('prepare')
@click.argument('osm_data_desc', type=str, callback=validators.validate_osm_data_desc)
@click.argument('output', type=str)
@click.option(
    '-c', '--config', type=click.File(), callback=validators.validate_json_file,
    help='Config file holding bound box extracts'
)
@click.option(
    '-d', '--dry-run', is_flag=True,
    help='Do not run any commands, just print what would be performed to stdout'
)
@click.option(
    '-s', '--silent', is_flag=True, default=False,
    help='Display no stdout output'
)
def prepare(osm_data_desc, output, config, dry_run, silent):
    """
    Retrieves an OSM data file and runs an optional extract-and-merge step to provide us
    with our project data
    """
    # Download the file to cache
    cache_file = cache.get_cache_file_from_url(osm_data_desc)

    if not cache_file.exists() and not dry_run:
        http.download_url(osm_data_desc, cache_file, show_progress=not silent)

    if dry_run is True:
        click.echo(click.style(f'Dry run: downloading {osm_data_desc} to {cache_file}', 'yellow'))

    # Extract-and-merge
    if config:
        extracts = config.get('extracts', [])
        osm.extract_multiple(cache_file, extracts, dry_run=dry_run, silent=silent, output=output)
    else:
        shutil.copy(cache_file, output)
