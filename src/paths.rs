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

/// A single path component usable as a new file or folder name. Rejects the
/// empty string, the dot entries, and anything containing a separator so the
/// name cannot escape the directory it is created in.
pub(crate) fn is_valid_entry_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\0')
}

/// Rewrites `path` when it is `from` or lives under it, mapping the prefix to
/// `to`. Returns `None` for unrelated paths so callers can keep them as-is.
pub(crate) fn remap_prefix(path: &Path, from: &Path, to: &Path) -> Option<PathBuf> {
    if path == from {
        return Some(to.to_path_buf());
    }
    path.strip_prefix(from).ok().map(|rest| to.join(rest))
}

/// Path shown relative to `root` when possible, otherwise absolute. Used by
/// "Copy Relative Path".
pub(crate) fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Copies a file, or a directory tree, to `to`. Directories are created and
/// filled recursively.
pub(crate) fn copy_entry(from: &Path, to: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(from)?;
    if metadata.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_entry(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else {
        std::fs::copy(from, to)?;
    }
    Ok(())
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

    #[test]
    fn valid_entry_names_reject_separators_and_dots() {
        assert!(is_valid_entry_name("main.rs"));
        assert!(is_valid_entry_name("My Notes"));
        assert!(!is_valid_entry_name(""));
        assert!(!is_valid_entry_name("."));
        assert!(!is_valid_entry_name(".."));
        assert!(!is_valid_entry_name("a/b"));
        assert!(!is_valid_entry_name("a\0b"));
    }

    #[test]
    fn remap_prefix_maps_self_and_descendants_only() {
        let from = Path::new("/tmp/project/src");
        let to = Path::new("/tmp/project/lib");
        assert_eq!(
            remap_prefix(from, from, to),
            Some(PathBuf::from("/tmp/project/lib"))
        );
        assert_eq!(
            remap_prefix(Path::new("/tmp/project/src/main.rs"), from, to),
            Some(PathBuf::from("/tmp/project/lib/main.rs"))
        );
        assert_eq!(
            remap_prefix(Path::new("/tmp/project/other"), from, to),
            None
        );
    }

    #[test]
    fn relative_display_strips_root_when_possible() {
        assert_eq!(
            relative_display(
                Path::new("/tmp/project/src/main.rs"),
                Path::new("/tmp/project")
            ),
            "src/main.rs"
        );
        assert_eq!(
            relative_display(Path::new("/elsewhere/main.rs"), Path::new("/tmp/project")),
            "/elsewhere/main.rs"
        );
    }

    #[test]
    fn copy_entry_duplicates_a_directory_tree() {
        let root = std::env::temp_dir().join(format!("tinytext-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let from = root.join("from");
        std::fs::create_dir_all(from.join("nested")).unwrap();
        std::fs::write(from.join("a.txt"), "a").unwrap();
        std::fs::write(from.join("nested/b.txt"), "b").unwrap();

        let to = root.join("to");
        copy_entry(&from, &to).unwrap();

        assert_eq!(std::fs::read_to_string(to.join("a.txt")).unwrap(), "a");
        assert_eq!(
            std::fs::read_to_string(to.join("nested/b.txt")).unwrap(),
            "b"
        );

        std::fs::remove_dir_all(&root).unwrap();
    }
}
