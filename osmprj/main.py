import click

from osmprj.commands.prepare import prepare
from osmprj.commands.report import report


@click.group()
@click.pass_context
def cli(ctx):
    ctx.ensure_object(dict)
    ctx.obj['config'] = {}


cli.add_command(report)
cli.add_command(prepare)

if __name__ == "__main__":
    cli()
