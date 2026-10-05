use std::path::{Path, PathBuf};

const BUNDLE_ID: &str = "net.nilambar.tinytext";

/// `~/Library/Application Support/<bundle id>`, home of session and settings files.
pub(crate) fn support_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(BUNDLE_ID),
    )
}

pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

pub(crate) fn read_dir(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }

    dirs.sort();
    files.sort();
    dirs.extend(files);
    dirs
}

pub(crate) fn path_from_file_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    path.starts_with('/')
        .then(|| PathBuf::from(percent_decode(path)))
}

pub(crate) fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_digit(bytes[index + 1]), hex_digit(bytes[index + 2]))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

pub(crate) fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_returns_last_component() {
        assert_eq!(file_name(Path::new("/tmp/project/main.rs")), "main.rs");
        assert_eq!(file_name(Path::new("/")), "/");
    }

    #[test]
    fn read_dir_lists_directories_before_files_sorted() {
        let root = std::env::temp_dir().join(format!("tinytext-read-dir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("zeta")).unwrap();
        std::fs::create_dir_all(root.join("alpha")).unwrap();
        std::fs::write(root.join("b.txt"), "").unwrap();
        std::fs::write(root.join("a.txt"), "").unwrap();

        let names: Vec<String> = read_dir(&root).iter().map(|path| file_name(path)).collect();
        std::fs::remove_dir_all(&root).unwrap();

        assert_eq!(names, ["alpha", "zeta", "a.txt", "b.txt"]);
    }

    #[test]
    fn read_dir_returns_empty_for_missing_directory() {
        assert!(read_dir(Path::new("/nonexistent/tinytext")).is_empty());
    }

    #[test]
    fn path_from_file_url_parses_local_paths() {
        assert_eq!(
            path_from_file_url("file:///Users/me/notes.txt"),
            Some(PathBuf::from("/Users/me/notes.txt"))
        );
        assert_eq!(
            path_from_file_url("file://localhost/Users/me/notes.txt"),
            Some(PathBuf::from("/Users/me/notes.txt"))
        );
        assert_eq!(
            path_from_file_url("file:///Users/me/My%20Notes.txt"),
            Some(PathBuf::from("/Users/me/My Notes.txt"))
        );
    }

    #[test]
    fn path_from_file_url_rejects_non_file_urls() {
        assert_eq!(path_from_file_url("https://example.com/a.txt"), None);
        assert_eq!(path_from_file_url("file://remote-host/a.txt"), None);
    }

    #[test]
    fn percent_decode_handles_valid_and_invalid_escapes() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("%2f%2F"), "//");
        assert_eq!(percent_decode("%E2%9C%93"), "✓");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("%2"), "%2");
    }
}
