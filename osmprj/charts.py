from typing import Sequence, Optional, NamedTuple

from rich.console import Console
from rich.table import Table
import plotly.graph_objects as go


def create_bar_chart(
        data: Sequence,
        x: str,
        y: str,
        output_file: str = 'chart.html',
        title: str = 'Bar Chart',
        xaxis_title: Optional[str] = None,
        yaxis_title: Optional[str] = None
) -> None:
    """
    Creates a bar chart and saves it to disk.
    """
    fig = go.Figure(go.Bar(
        x=[getattr(row, x) for row in data],
        y=[getattr(row, y) for row in data],
        orientation='h',
        texttemplate='%{x:.2f}',
    ))
    xaxis_title = xaxis_title or x
    yaxis_title = yaxis_title or y

    fig.update_layout(
        title=title,
        xaxis_title=xaxis_title,
        yaxis_title=yaxis_title,
        font=dict(
            family="Lora",
            size=32,
            color="#666"
        ),
        xaxis=dict(
            color='#999'
        ),
        yaxis=dict(
            color='#666',
            tickfont=dict(
                size=36
            ),
            title=dict(
                font=dict(
                    color='#999'
                )
            )
        )
    )
    fig.update_traces(marker_color='#45818e')
    fig.write_html(output_file)


def print_table(data: Sequence[NamedTuple], title: str, fields: dict) -> None:
    """Uses rich to print a table output"""
    table = Table(title=title)
    display_funcs = tuple(
        fld.get('display_func', lambda x: x)
        for key, fld in fields.items()
    )

    for key, fld in fields.items():
        table.add_column(fld['display_name'], style=fld['color'])

    for row in data:
        row_str = tuple(
            d_func(fld) for fld, d_func in zip(row, display_funcs)
        )
        table.add_row(*row_str)

    console = Console()
    console.print(table)
