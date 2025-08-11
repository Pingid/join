use std::{collections::HashSet, env, fs, path::PathBuf, process};

fn usage(p: &str) -> String {
    format!(
        "Usage: {p} <FILE> [FILE...]\n\nConcatenate files to stdout.\n\nExamples:\n  {p} a.txt b.md"
    )
}

fn main() {
    let mut a = env::args();
    if a.len() == 1 {
        println!("{}", usage(&a.next().unwrap()));
        process::exit(1);
    }
    a.next();

    let mut visited = HashSet::new();

    for path in a {
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
        print!("file: {:?}\n{contents}\n", path);
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
