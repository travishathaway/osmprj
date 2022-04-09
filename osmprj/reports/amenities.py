import pathlib
from functools import wraps
from typing import Sequence, NamedTuple

from psycopg2.extensions import cursor as Cursor  # no-qa

REPORT_SQL_DIR = pathlib.Path(__file__).parent.absolute().joinpath('sql')


def get_sql_script(script, params: dict):
    """decorator for opening and reading in alias values for sql script"""
    def wrap(f):
        @wraps(f)
        def wrapper(*args, **kwargs):
            sql_file = REPORT_SQL_DIR.joinpath(script)

            with open(sql_file) as fp:
                sql_str = fp.read()
            sql_aliases = {key: fld['name'] for key, fld in params.items()}
            sql_str = sql_str.format(**sql_aliases)

            return f(sql_str, *args, **kwargs)
        return wrapper
    return wrap


AMENITY_DATA_FIELDS = {
    'city': {
        'name': 'city',
        'display_name': 'City',
        'color': 'green'
    },
    'amenity': {
        'name': 'amenity',
        'display_name': 'Amenity',
        'color': 'white'
    },
    'area_sq_km': {
        'name': 'area_sq_km',
        'display_name': 'Area (sq. km)',
        'color': 'cyan',
        'display_func': lambda x: str(round(x, 2))
    },
    'count': {
        'name': 'count',
        'display_name': '# of amenities',
        'color': 'cyan',
        'display_func': str
    },
    'amenity_per_sq_km': {
        'name': 'amenity_per_sq_km',
        'display_name': 'Amenities per sq. km',
        'color': 'cyan',
        'display_func': lambda x: str(round(x, 2))
    },
}


@get_sql_script('amenity_counts_by_city.sql', AMENITY_DATA_FIELDS)
def get_amenity_data_by_city(sql: str, cursor: Cursor, cities: Sequence[str], amenity: str) -> Sequence[NamedTuple]:
    """
    Grab the count of amenity for a list of cities.
    """
    params = {
        'amenity': amenity,
        'cities': cities
    }
    cursor.execute(sql, params)

    return cursor.fetchall()


PARKING_DATA_FIELDS = {
    'city': {
        'name': 'city',
        'display_name': 'City',
        'color': 'green'
    },
    'parking_area_sq_km': {
        'name': 'parking_area_sq_km',
        'display_name': 'Total parking area (sq. km)',
        'color': 'cyan',
        'display_func': lambda x: str(round(x, 2))
    },
    'city_area_sq_km': {
        'name': 'city_area_sq_km',
        'display_name': 'City area (sq. km)',
        'color': 'cyan',
        'display_func': lambda x: str(round(x, 2))
    },
    'percentage_parking_area': {
        'name': 'percentage_parking_area',
        'display_name': '% Parking area',
        'color': 'cyan',
        'display_func': lambda x: f'{round(x, 4)}%'
    }
}


@get_sql_script('parking_space_by_city.sql', PARKING_DATA_FIELDS)
def get_parking_area_by_city(sql: str, cursor: Cursor, cities: Sequence[str]) -> Sequence[NamedTuple]:
    """
    Calculate the "parking area" per square kilometer for provided cities.
    """
    params = {
       'cities': cities
    }
    cursor.execute(sql, params)

    return cursor.fetchall()
