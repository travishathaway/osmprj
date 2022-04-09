import json

import click

from osmprj import http
from osmprj import cache


def validate_json_file(_, __, value):
    """
    Ensures we have a valid JSON file before continuing.
    """
    if value:
        try:
            return json.load(value)
        except json.JSONDecodeError as exc:
            raise click.BadArgumentUsage(f'Error decoding JSON: {exc}')


def validate_osm_data_desc(_, __, value):
    url = http.build_url_from_desc(value)

    cache_file = cache.get_cache_file_from_url(url)

    if not cache_file.exists() and not http.is_valid_resource(url):
        raise click.ClickException('Not a valid GeoFabrik resource')

    return url


OUTPUT_TYPE_TERMINAL = 'terminal'
OUTPUT_TYPE_CHART = 'chart'
OUTPUT_TYPE_CHOICES = (OUTPUT_TYPE_CHART, OUTPUT_TYPE_TERMINAL)


def validate_output_type(_, __, value):
    if value not in OUTPUT_TYPE_CHOICES:
        raise click.ClickException(f'Not a valid output-type, choices are: {OUTPUT_TYPE_CHOICES}')
    return value
