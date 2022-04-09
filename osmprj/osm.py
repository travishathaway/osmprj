import subprocess
from pathlib import Path
from typing import Iterable, Sequence

import click

from osmprj import cache
from osmprj.types import OSMCityData


def run_command(command: Sequence[str], dry_run: bool = False, silent: bool = False):
    """Function which either runs the command or performs a dry run (i.e. print info to stdout)"""
    if not silent and not dry_run:
        click.echo(f"{click.style('Running Command:', fg='green')} {' '.join(command)}")
    if not dry_run:
        try:
            subprocess.run(command)
        except FileNotFoundError:
            raise click.ClickException(f'Unable to find {command[0] if command else "None"}')
    else:
        click.echo(click.style(f'Dry run:  {" ".join(command)}', 'yellow'))


def get_extract_file_name(osm_file: Path, config: OSMCityData) -> Path:
    """
    Get the cache file name which incorporates bbox into the name
    """
    outfile = Path(config['output'])
    bbox_str = '-'.join([str(x) for x in config['bbox']])

    cache_dir = osm_file.parent.joinpath(osm_file.name.rstrip('.osm.pbf'))
    cache.create_cache_dir(cache_dir)  # only creates it if it doesn't already exist

    return cache_dir.joinpath(f'{outfile.name.rstrip(".osm.pbf")}-{bbox_str}.osm.pbf')


def extract_multiple(
        osm_file: Path,
        extracts: Sequence[OSMCityData],
        output: str = 'project-data.osm.pbf',
        dry_run: bool = False,
        silent: bool = False
):
    """
    Provided an `osm_file` and `extracts`, extract the data from the `osm_file`
    and merge it into the `output` file.
    """
    cache_filenames = tuple(
        get_extract_file_name(osm_file, config) for config in extracts
    )
    cached_extracts = tuple(
        (file, None) for file in cache_filenames if file.exists()
    )
    new_extracts = tuple(
        (file, ext)
        for file, ext in zip(cache_filenames, extracts)
        if not file.exists()
    )
    commands = tuple(
        extract(osm_file, ext_out, ext_config)
        for ext_out, ext_config in new_extracts
    )
    # Create the extracts
    tuple(
        run_command(command, dry_run=dry_run, silent=silent)
        for command in commands
    )

    # Merge them all in to one file
    output_extracts = (filename for filename, *_ in new_extracts + cached_extracts)

    run_command(
        merge(output_extracts, output=output),
        dry_run=dry_run,
        silent=silent
    )


def extract(
        osm_file: Path,
        output: Path,
        city_config: OSMCityData,
) -> Sequence[str]:
    """
    Given a `city_config` object, extract the contents of `osmfile` according to the `bbox` property.
    """
    buffer = 0.05
    x_min, y_min, x_max, y_max = city_config['bbox']
    bbox = x_min - buffer, y_min - buffer, x_max + buffer, y_max + buffer
    bbox_strings = (str(coord) for coord in bbox)
    bbox_str = ','.join(bbox_strings)

    return [
        'osmium',
        'extract',
        '--overwrite',
        '--bbox',
        bbox_str,
        '--output',
        str(output),
        str(osm_file),
    ]


def merge(
        osm_files: Iterable[Path],
        output='project-data.osm.pbf'
) -> Sequence[str]:
    """
    Provides a thin wrapper around the `osmium merge` command
    """
    command = [
        'osmium',
        'merge'
    ]
    command += [
        str(filename) for filename in osm_files
    ]
    command += [
        '--overwrite',
        '--output',
        output
    ]

    return command
