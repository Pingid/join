use std::{
    collections::HashSet,
    env, fs,
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
    process,
};

fn usage(p: &str) -> String {
    format!(
        "Usage: {p} <FILE> [FILE...]\n\nConcatenate files to stdout.\n\nExamples:\n  {p} a.txt b.md"
    )
}

fn main() {
    // Collect args (excluding program name)
    let mut args = env::args();
    let prog = args.next().unwrap_or_else(|| "join".into());
    let mut inputs: Vec<String> = args.collect();

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

    let mut visited = HashSet::new();

    for path in inputs {
        match print_entry(&mut visited, path.clone().into()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("join: Error reading {path:?}: {e}");
                process::exit(1);
            }
        }
    }
}

fn print_entry(visited: &mut HashSet<PathBuf>, path: PathBuf) -> Result<(), std::io::Error> {
    if visited.contains(&path) {
        return Ok(());
    }
    visited.insert(path.clone());
    let meta = fs::metadata(&path)?;

    if meta.is_file() {
        let contents = fs::read_to_string(&path)?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        print!("### {path:?}\n```{ext}\n{contents}\n```\n");
        return Ok(());
    }

    let entries = fs::read_dir(&path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        print_entry(visited, path)?;
    }
    Ok(())
}
