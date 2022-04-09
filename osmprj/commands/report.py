import json

import click

from osmprj import validators
from osmprj.charts import create_bar_chart, print_table
from osmprj.db import psycopg2_cursor
from osmprj.reports.amenities import (
    get_amenity_data_by_city,
    get_parking_area_by_city,
    AMENITY_DATA_FIELDS,
    PARKING_DATA_FIELDS,
)


@click.group()
@click.option('-c', '--config', type=click.File(), help=f'Config file holding database connection information')
@click.pass_context
def report(ctx, config):
    """
    These sub-commands are responsible for generating reports
    """
    if config is None:
        raise click.ClickException('Please provide a config file for the -c/--config option')
    try:
        config_data = json.load(config)
    except Exception as exc:
        raise click.ClickException(f'Could not parse JSON config: {exc}')

    try:
        ctx.obj['pg_dsn'] = config_data['pg_dsn']
    except KeyError:
        raise click.ClickException('pg_dsn cannot be empty in config file')


@click.command('amenity_city')
@click.argument('cities', type=str)
@click.argument('amenity', type=str)
@click.option(
    '-o',
    '--output-type',
    default=validators.OUTPUT_TYPE_TERMINAL,
    help=f'Possible choices: {", ".join(validators.OUTPUT_TYPE_CHOICES)}',
    callback=validators.validate_output_type
)
@click.option('-f', '--output-file', default='chart.html', help='Filename to use when output-type is chart')
@click.pass_context
def amenity_count_by_city(ctx, cities, amenity, output_type, output_file):
    cities = tuple(cty.strip() for cty in cities.split(','))

    with psycopg2_cursor(ctx.obj['pg_dsn']) as cursor:
        data = get_amenity_data_by_city(cursor, cities, amenity)

    if output_type == validators.OUTPUT_TYPE_TERMINAL:
        print_table(data, title=f'Amenity county by city: {amenity}', fields=AMENITY_DATA_FIELDS)

    elif output_type == validators.OUTPUT_TYPE_CHART:
        create_bar_chart(
            data,
            x=AMENITY_DATA_FIELDS['amenity_per_sq_km']['name'],
            y=AMENITY_DATA_FIELDS['city']['name'],
            title=f'Amenity count by city: {amenity}',
            xaxis_title='Amenity per sq. km',
            yaxis_title='Top 10 German cities (by pop.)',
            output_file=output_file
        )


@click.command('parking_space')
@click.argument('cities', type=str)
@click.option(
    '-o',
    '--output-type',
    default=validators.OUTPUT_TYPE_TERMINAL,
    help=f'Possible choices: {", ".join(validators.OUTPUT_TYPE_CHOICES)}',
    callback=validators.validate_output_type
)
@click.pass_context
def parking_space_by_city(ctx, cities, output_type):
    cities = tuple(cty.strip() for cty in cities.split(','))

    with psycopg2_cursor(ctx.obj['pg_dsn']) as cursor:
        data = get_parking_area_by_city(cursor, cities)

    if output_type == validators.OUTPUT_TYPE_TERMINAL:
        print_table(data, title='Parking Area', fields=PARKING_DATA_FIELDS)

    elif output_type == validators.OUTPUT_TYPE_CHART:
        create_bar_chart(
            data,
            x=PARKING_DATA_FIELDS['percentage_parking_area']['name'],
            y=PARKING_DATA_FIELDS['city']['name'],
            title=f'Percentage parking area by city',
            xaxis_title='Percentage Parking area',
            yaxis_title='Top 10 German cities (by pop.)'
        )


report.add_command(amenity_count_by_city)
report.add_command(parking_space_by_city)
