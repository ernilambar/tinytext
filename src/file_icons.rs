//! Material Icon Theme file icons.
//!
//! The SVGs are vendored from the VS Code Material Icon Theme (MIT); see
//! `assets/file-icons/LICENSE`. The name and extension associations mirror that
//! theme's generated manifest.

use std::path::Path;

macro_rules! icon {
    ($name:literal) => {
        include_bytes!(concat!("../assets/file-icons/", $name, ".svg")) as &[u8]
    };
}

const DEFAULT_FILE: &[u8] = icon!("file");
const DEFAULT_FOLDER: &[u8] = icon!("folder");

pub(crate) fn folder_icon() -> &'static [u8] {
    DEFAULT_FOLDER
}

pub(crate) fn file_icon(path: &Path) -> &'static [u8] {
    let by_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .and_then(|name| icon_for_name(&name));
    if let Some(icon) = by_name {
        return icon;
    }

    let by_extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .and_then(|ext| icon_for_extension(&ext));
    by_extension.unwrap_or(DEFAULT_FILE)
}

fn icon_for_name(name: &str) -> Option<&'static [u8]> {
    let icon: &'static [u8] = match name {
        ".gitignore" | ".gitignore_global" | ".gitattributes" | ".gitmodules" | ".gitconfig" => {
            icon!("git")
        }
        ".editorconfig" => icon!("editorconfig"),
        ".npmrc" => icon!("npm"),
        "package.json" | "package-lock.json" | "npm-shrinkwrap.json" => icon!("nodejs"),
        "tsconfig.json" => icon!("tsconfig"),
        ".eslintrc" | ".eslintrc.js" | ".eslintrc.cjs" | ".eslintrc.json" | ".eslintrc.yaml"
        | ".eslintrc.yml" | "eslint.config.js" | "eslint.config.mjs" | "eslint.config.cjs"
        | "eslint.config.ts" => icon!("eslint"),
        ".prettierrc"
        | ".prettierrc.json"
        | ".prettierrc.js"
        | ".prettierrc.cjs"
        | ".prettierrc.yaml"
        | ".prettierrc.yml"
        | "prettier.config.js"
        | "prettier.config.cjs" => {
            icon!("prettier")
        }
        ".babelrc" | ".babelrc.js" | ".babelrc.cjs" | "babel.config.js" | "babel.config.cjs"
        | "babel.config.json" => icon!("babel"),
        "vite.config.js" | "vite.config.ts" | "vite.config.mjs" | "vite.config.mts"
        | "vite.config.cjs" | "vite.config.cts" => icon!("vite"),
        "webpack.config.js"
        | "webpack.config.ts"
        | "webpack.config.mjs"
        | "webpack.config.cjs"
        | "webpack.config.babel.js" => icon!("webpack"),
        "makefile" | "makefile.am" | "makefile.in" | "gnumakefile" => icon!("makefile"),
        "settings.json" => icon!("settings"),
        "license" | "licence" | "copying" | "copyright" => icon!("license"),
        ".envrc" => icon!("console"),
        ".bashrc" | ".bash_profile" | ".bash_login" | ".bash_aliases" | ".profile" | ".zshrc"
        | ".zprofile" | ".zshenv" | ".zlogin" | ".kshrc" | ".cshrc" | ".tcshrc" | ".inputrc" => {
            icon!("console")
        }
        _ => {
            if name.starts_with("dockerfile") || name.starts_with("docker-compose") {
                return Some(icon!("docker"));
            }
            if name.starts_with("readme") {
                return Some(icon!("readme"));
            }
            if name.starts_with("license") || name.starts_with("licence") {
                return Some(icon!("license"));
            }
            if name.starts_with(".env") {
                return Some(icon!("tune"));
            }
            return None;
        }
    };
    Some(icon)
}

fn icon_for_extension(ext: &str) -> Option<&'static [u8]> {
    let icon: &'static [u8] = match ext {
        "rs" => icon!("rust"),
        "js" | "mjs" | "cjs" => icon!("javascript"),
        "jsx" => icon!("react"),
        "ts" | "mts" | "cts" => icon!("typescript"),
        "tsx" => icon!("react_ts"),
        "vue" => icon!("vue"),
        "svelte" => icon!("svelte"),
        "astro" => icon!("astro"),
        "py" | "pyi" | "pyw" => icon!("python"),
        "php" | "phtml" | "php3" | "php4" | "php5" => icon!("php"),
        "html" | "htm" => icon!("html"),
        "css" => icon!("css"),
        "scss" | "sass" => icon!("sass"),
        "less" => icon!("less"),
        "styl" | "stylus" => icon!("stylus"),
        "coffee" => icon!("coffee"),
        "ls" => icon!("livescript"),
        "c" => icon!("c"),
        "h" => icon!("h"),
        "cpp" | "cc" | "cxx" | "c++" | "cp" => icon!("cpp"),
        "hpp" | "hh" | "hxx" | "h++" => icon!("hpp"),
        "cs" | "csx" => icon!("csharp"),
        "java" => icon!("java"),
        "kt" | "kts" => icon!("kotlin"),
        "swift" => icon!("swift"),
        "go" => icon!("go"),
        "rb" | "rake" | "gemspec" => icon!("ruby"),
        "lua" | "luau" => icon!("lua"),
        "dart" => icon!("dart"),
        "scala" | "sc" => icon!("scala"),
        "ex" | "exs" => icon!("elixir"),
        "erl" | "hrl" => icon!("erlang"),
        "hs" => icon!("haskell"),
        "clj" | "cljs" | "cljc" | "edn" => icon!("clojure"),
        "elm" => icon!("elm"),
        "nim" | "nims" => icon!("nim"),
        "zig" | "zon" => icon!("zig"),
        "jl" => icon!("julia"),
        "r" => icon!("r"),
        "asm" | "s" => icon!("assembly"),
        "f" | "f90" | "f95" | "f03" | "for" | "ftn" => icon!("fortran"),
        "m" => icon!("objective-c"),
        "ml" | "mli" => icon!("ocaml"),
        "fs" | "fsi" | "fsx" | "fsscript" => icon!("fsharp"),
        "purs" => icon!("purescript"),
        "re" | "rei" => icon!("reason"),
        "res" | "resi" => icon!("rescript"),
        "graphql" | "gql" => icon!("graphql"),
        "prisma" => icon!("prisma"),
        "sol" => icon!("solidity"),
        "tex" | "latex" | "sty" | "cls" => icon!("tex"),
        "tf" | "tfvars" | "tfstate" => icon!("terraform"),
        "nginx" | "nginxconf" => icon!("nginx"),
        "sh" | "bash" | "zsh" | "ksh" | "fish" | "ash" => icon!("console"),
        "ps1" | "psm1" | "psd1" => icon!("powershell"),
        "ipynb" => icon!("jupyter"),
        "j2" | "jinja" | "jinja2" => icon!("jinja"),
        "pug" | "jade" => icon!("pug"),
        "ejs" => icon!("ejs"),
        "twig" => icon!("twig"),
        "haml" => icon!("haml"),
        "razor" | "cshtml" => icon!("razor"),
        "json" | "jsonc" | "json5" | "jsonl" => icon!("json"),
        "yaml" | "yml" => icon!("yaml"),
        "toml" => icon!("toml"),
        "xml" | "xsd" | "xsl" | "xslt" | "dtd" | "plist" => icon!("xml"),
        "md" | "markdown" => icon!("markdown"),
        "mdx" => icon!("mdx"),
        "sql" | "db" | "sqlite" | "sqlite3" | "mysql" | "pgsql" => icon!("database"),
        "csv" | "tsv" => icon!("table"),
        "lock" => icon!("lock"),
        "key" | "pem" | "p12" | "pfx" => icon!("key"),
        "crt" | "cer" | "der" => icon!("certificate"),
        "svg" => icon!("svg"),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "avif" => {
            icon!("image")
        }
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => icon!("audio"),
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v" => icon!("video"),
        "ttf" | "otf" | "woff" | "woff2" | "eot" => icon!("font"),
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "rar" | "7z" | "zst" => icon!("zip"),
        "pdf" => icon!("pdf"),
        "doc" | "docx" | "odt" | "rtf" | "pages" => icon!("word"),
        "txt" => icon!("document"),
        "log" => icon!("log"),
        _ => return None,
    };
    Some(icon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_extensions() {
        for (file, expected) in [
            ("main.rs", icon!("rust")),
            ("app.js", icon!("javascript")),
            ("app.mjs", icon!("javascript")),
            ("component.jsx", icon!("react")),
            ("app.ts", icon!("typescript")),
            ("view.tsx", icon!("react_ts")),
            ("index.html", icon!("html")),
            ("style.css", icon!("css")),
            ("style.scss", icon!("sass")),
            ("script.py", icon!("python")),
            ("index.php", icon!("php")),
            ("data.json", icon!("json")),
            ("config.yaml", icon!("yaml")),
            ("Cargo.toml", icon!("toml")),
            ("Cargo.lock", icon!("lock")),
            ("notes.txt", icon!("document")),
            ("photo.png", icon!("image")),
            ("archive.zip", icon!("zip")),
        ] {
            assert_eq!(file_icon(Path::new(file)), expected, "{file}");
        }
    }

    #[test]
    fn matches_special_file_names_before_extension() {
        for (file, expected) in [
            ("package.json", icon!("nodejs")),
            ("package-lock.json", icon!("nodejs")),
            ("tsconfig.json", icon!("tsconfig")),
            (".gitignore", icon!("git")),
            (".editorconfig", icon!("editorconfig")),
            ("Dockerfile", icon!("docker")),
            ("Dockerfile.prod", icon!("docker")),
            ("docker-compose.yml", icon!("docker")),
            ("Makefile", icon!("makefile")),
            ("README.md", icon!("readme")),
            ("LICENSE", icon!("license")),
            (".npmrc", icon!("npm")),
            (".env", icon!("tune")),
            (".bashrc", icon!("console")),
        ] {
            assert_eq!(file_icon(Path::new(file)), expected, "{file}");
        }
    }

    #[test]
    fn falls_back_to_default_file_icon() {
        for file in ["notes.unknownext", "Makefile.unknown", "noextension"] {
            assert_eq!(file_icon(Path::new(file)), DEFAULT_FILE, "{file}");
        }
    }
}
