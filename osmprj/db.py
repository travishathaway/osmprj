from contextlib import contextmanager

import psycopg2
from psycopg2.extras import NamedTupleCursor


@contextmanager
def psycopg2_cursor(pg_dsn: str):
    """
    Return a psycopg2 cursor as ContextManager
    """
    connection = psycopg2.connect(pg_dsn)

    try:
        cursor = connection.cursor(cursor_factory=NamedTupleCursor)
        yield cursor
    finally:
        connection.commit()
        connection.close()
