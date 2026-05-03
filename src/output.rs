use clap::builder::styling::{AnsiColor, Style, Styles};
use console::{style, StyledObject};
use indicatif::ProgressStyle;

pub fn icon_success() -> StyledObject<&'static str> {
    style("✓").green()
}

pub fn icon_error() -> StyledObject<&'static str> {
    style("✗").red()
}

pub fn icon_warn() -> StyledObject<&'static str> {
    style("⚠").yellow()
}

pub fn icon_info() -> StyledObject<&'static str> {
    style("ℹ").dim()
}

pub fn icon_skip() -> StyledObject<&'static str> {
    style("⊙").dim()
}

pub fn progress_bar_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "  {spinner:.cyan} {msg:<35} [{bar:40.green/white}] {bytes}/{total_bytes} ({bytes_per_sec}, eta {eta})",
    )
    .unwrap()
    .progress_chars("█▓░")
}

pub fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner} {msg}")
        .unwrap()
        .tick_strings(&["🌍 ", "🌎 ", "🌏 ", "🌐 ", "🌍 "])
}

pub fn help_styles() -> Styles {
    Styles::styled()
        .header(Style::new().bold().underline())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(Style::new().bold())
        .placeholder(Style::new().dimmed())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Yellow.on_default())
}
