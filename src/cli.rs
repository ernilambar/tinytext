use std::path::{Path, PathBuf};

pub(crate) fn app_bundle_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
}

pub(crate) fn launcher_script(bundle: &Path) -> String {
    let template = r#"#!/usr/bin/env bash
# Command-line launcher for Tinytext, installed by the app.
set -euo pipefail

APP="${TINYTEXT_APP:-__BUNDLE__}"

if [ "$#" -eq 0 ]; then
    exec open -a "$APP"
fi

exec open -a "$APP" -- "$@"
"#;
    template.replace("__BUNDLE__", &bundle.display().to_string())
}

pub(crate) fn install_cli(bundle: &Path) -> Result<PathBuf, String> {
    const TARGET: &str = "/usr/local/bin/tinytext";

    let temp = std::env::temp_dir().join(format!("tinytext-cli-{}", std::process::id()));
    std::fs::write(&temp, launcher_script(bundle)).map_err(|error| error.to_string())?;

    let command = format!(
        "mkdir -p /usr/local/bin && install -m 755 \"{}\" {TARGET}",
        temp.display()
    );
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        command.replace('"', "\\\"")
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|error| error.to_string())?;

    let _ = std::fs::remove_file(&temp);

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        let message = message.trim();
        let message = message.strip_prefix("execution error: ").unwrap_or(message);
        if message.contains("User canceled") || message.contains("-128") {
            return Err("Installation cancelled".to_string());
        }
        return Err(message.to_string());
    }

    Ok(PathBuf::from(TARGET))
}

pub(crate) fn print_help() {
    println!(
        "\
{name} {version}
A native macOS text editor built with GPUI Kit

Usage:
  tinytext [OPTIONS] [FOLDER]

Arguments:
  [FOLDER]  Open the given folder in the sidebar

Options:
  -h, --help     Print this help and exit
  -V, --version  Print version and exit",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_script_embeds_bundle_path() {
        let script = launcher_script(Path::new("/Applications/Tinytext.app"));
        assert!(script.starts_with("#!/usr/bin/env bash\n"));
        assert!(script.contains(r#"APP="${TINYTEXT_APP:-/Applications/Tinytext.app}""#));
        assert!(!script.contains("__BUNDLE__"));
    }
}
