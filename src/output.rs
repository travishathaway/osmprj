use clap::builder::styling::{AnsiColor, Style, Styles};
use console::{style, StyledObject, Term};
use indicatif::ProgressStyle;

const WIDE_BREAKPOINT: u16 = 80;
const WIDE_MSG_WIDTH: usize = 25;
const NARROW_MSG_WIDTH: usize = 15;

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

/// Returns the max message width that matches the current terminal's progress bar template.
pub fn progress_bar_msg_width() -> usize {
    let (_, cols) = Term::stderr().size();
    if cols >= WIDE_BREAKPOINT {
        WIDE_MSG_WIDTH
    } else {
        NARROW_MSG_WIDTH
    }
}

/// Truncates `msg` to `max_len` characters, appending `…` if truncated.
pub fn truncate_message(msg: &str, max_len: usize) -> String {
    let char_count = msg.chars().count();
    if char_count <= max_len {
        msg.to_string()
    } else {
        let truncated: String = msg.chars().take(max_len.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

pub fn progress_bar_style() -> ProgressStyle {
    let (_, cols) = Term::stderr().size();
    if cols >= WIDE_BREAKPOINT {
        ProgressStyle::with_template(
            "  {spinner:.cyan} {msg:<25} [{bar:30.green/white}] {bytes}/{total_bytes} (eta {eta})",
        )
        .unwrap()
        .progress_chars("█▓░")
    } else {
        ProgressStyle::with_template(
            "  {spinner:.cyan} {msg:<15} [{bar:15.green/white}] {bytes}/{total_bytes}",
        )
        .unwrap()
        .progress_chars("█▓░")
    }
}

/// Style for a download that is queued but waiting for a semaphore permit.
/// Shows a dim spinner and "Pending <name>" with no progress bar or byte counts.
pub fn pending_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.dim} {msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
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
