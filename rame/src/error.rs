use std::fmt::Display;
use std::path::Path;

pub(crate) fn format_err(path: &Path, line: &str, line_nb: usize, msg: impl Display) -> String {
    if !line.is_empty() {
        format!("error: {}:{}: {:?}: {}", path.display(), line_nb + 1, line, msg)
    } else {
        format!("error: {}:{}: {}", path.display(), line_nb + 1, msg)
    }
}

pub(crate) fn format_help(path: &Path, line_nb: usize, msg: impl Display) -> String {
    format_err(path, "", line_nb, format!("help: {msg}"))
}
