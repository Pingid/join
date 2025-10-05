mod rust;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use pico_args::Arguments;
use std::{
    collections::HashSet,
    env, fs,
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
    process,
};

fn usage(p: &str) -> String {
    format!(
        r#"Usage: {p} [--exclude|-e <GLOB>] [--no-default-excludes|-E] [--strip-rust-tests] <FILE> [FILE...]

Concatenate files to markdown.

Options:
  --exclude, -e <GLOB>        Exclude paths matching the glob (repeatable).
  --no-default-excludes, -E   Disable built-in default excludes.
  --strip-rust-tests          Strip test modules and test functions from .rs files.
  --version, -V               Print version and exit.

Examples:
  {p} a.txt b.md
  {p} -e 'target/*' -e '*.{{json,md}}' .
  {p} -E .
  {p} --strip-rust-tests src/
  {p} printf "a.txt\nb.md\n" | join
  "#
    )
}

// Default excludes applied globally, can be augmented with --exclude/-e
// Dotfiles & dot-directories are handled separately via `has_dot_component`.
const EXCLUDE: [&str; 25] = [
    // Common dependency/build dirs
    "*/target/*",
    "*/node_modules/*",
    "*/dist/*",
    "*/build/*",
    "*/out/*",
    "*/coverage/*",
    "*/vendor/*",
    "*/__pycache__/*",
    // Lock files
    "*Cargo.lock",
    "*package-lock.json",
    "*yarn.lock",
    "*pnpm-lock.yaml",
    "*bun.lock",
    "*Pipfile.lock",
    "*poetry.lock",
    "*Gemfile.lock",
    "*composer.lock",
    "*go.sum",
    "*npm-shrinkwrap.json",
    // Compiled/binary-ish artifacts and logs
    "*.pyc",
    "*.pyo",
    "*.pyd",
    "*.log",
    // OS/editor cruft
    "*Thumbs.db",
    "*Desktop.ini",
];

fn main() {
    // Collect args (excluding program name)
    let mut args_iter = env::args();
    let prog = args_iter.next().unwrap_or_else(|| "join".into());

    // Parse options and inputs with pico-args
    let mut pargs = Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        println!("{}", usage(&prog));
        process::exit(0);
    }

    // Version flag
    if pargs.contains("--version") || pargs.contains("-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    // Optional flag to disable built-in default excludes
    let no_default_excludes = pargs.contains("--no-default-excludes") || pargs.contains("-E");
    let defaults_enabled = !no_default_excludes;

    // Optional flag to strip Rust tests
    let strip_rust_tests = pargs.contains("--strip-rust-tests");

    // Support repeatable --exclude and -e
    let mut exclude = GlobSetBuilder::new();
    let xs: Vec<String> = pargs.values_from_str("--exclude").unwrap_or_else(|e| {
        eprintln!("{prog}: --exclude expects a value: {e}");
        println!("{}", usage(&prog));
        process::exit(1);
    });

    for pattern in xs {
        let g = GlobBuilder::new(&pattern)
            .literal_separator(false)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("{prog}: invalid glob pattern in --exclude: {e}");
                println!("{}", usage(&prog));
                process::exit(1);
            });
        exclude.add(g);
    }

    let ys: Vec<String> = pargs.values_from_str("-e").unwrap_or_else(|e| {
        eprintln!("{prog}: -e expects a value: {e}");
        println!("{}", usage(&prog));
        process::exit(1);
    });
    for pattern in ys {
        let g = GlobBuilder::new(&pattern)
            .literal_separator(false)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("{prog}: invalid glob pattern in -e: {e}");
                println!("{}", usage(&prog));
                process::exit(1);
            });
        exclude.add(g);
    }

    let mut inputs: Vec<String> = pargs
        .finish()
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();

    // If stdin is piped, read additional inputs (newline-separated paths)
    if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(s) => {
                    let s = s.trim();
                    if !s.is_empty() {
                        inputs.push(s.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("join: Error reading stdin: {e}");
                    process::exit(1);
                }
            }
        }
    }

    // If no inputs from args or stdin, show usage
    if inputs.is_empty() {
        println!("{}", usage(&prog));
        process::exit(1);
    }

    // Combine default excludes with CLI-provided ones
    if !no_default_excludes {
        for s in EXCLUDE {
            let g = GlobBuilder::new(s)
                .literal_separator(false)
                .build()
                .unwrap();
            exclude.add(g);
        }
    }

    let excludes = exclude.build().unwrap_or_else(|e| {
        eprintln!("{prog}: invalid glob pattern: {e}");
        println!("{}", usage(&prog));
        process::exit(1);
    });

    let mut visited = HashSet::new();

    for path in inputs {
        match print_entry(
            &mut visited,
            path.clone().into(),
            &excludes,
            defaults_enabled,
            strip_rust_tests,
        ) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("join: Error reading {path:?}: {e}");
                process::exit(1);
            }
        }
    }
}

fn print_entry(
    visited: &mut HashSet<PathBuf>,
    path: PathBuf,
    excludes: &GlobSet,
    defaults_enabled: bool,
    strip_rust_tests: bool,
) -> Result<(), std::io::Error> {
    if is_excluded(&path, excludes, defaults_enabled) {
        return Ok(());
    }
    if visited.contains(&path) {
        return Ok(());
    }
    visited.insert(path.clone());
    let meta = fs::metadata(&path)?;

    if meta.is_file() {
        let contents = fs::read_to_string(&path);
        match contents {
            Err(e) => {
                eprintln!("Error {path:?}: {e}");
                return Ok(());
            }
            Ok(contents) => {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let processed_contents = if strip_rust_tests && ext == "rs" {
                    rust::strip_tests(&contents)
                } else {
                    contents
                };
                print!("### {path:?}\n```{ext}\n{processed_contents}\n```\n");
                return Ok(());
            }
        };
    }

    let entries = fs::read_dir(&path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        print_entry(visited, path, excludes, defaults_enabled, strip_rust_tests)?;
    }
    Ok(())
}

fn is_excluded(path: &std::path::Path, excludes: &GlobSet, defaults_enabled: bool) -> bool {
    if excludes.is_match(path) {
        return true;
    }
    if defaults_enabled && has_dot_component(path) {
        return true;
    }
    false
}

fn has_dot_component(path: &std::path::Path) -> bool {
    use std::ffi::OsStr;
    for comp in path.components() {
        if let std::path::Component::Normal(os) = comp {
            if let Some(s) = os.to_str() {
                if s.starts_with('.') {
                    return true;
                }
            } else if os == OsStr::new(".") {
                return true;
            }
        }
    }
    false
}
