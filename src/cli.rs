use clap::{Arg, ArgAction, Command};
use globset::{GlobBuilder, GlobSetBuilder};
use std::{fs, process};
use std::{
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
};

use crate::lang::{LangContext, specs};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("join")
        .about("Concatenate files to markdown")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .value_name("GLOB")
                .help("Exclude paths matching the glob (repeatable)")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("no-default-excludes")
                .short('E')
                .long("no-default-excludes")
                .help("Disable built-in default excludes")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("inputs")
                .value_name("FILE")
                .help("Input files or directories")
                .num_args(0..),
        );

    for spec in specs() {
        for arg in &spec.args {
            cmd = cmd.arg(arg.clone());
        }
    }

    let matches = cmd.clone().get_matches();

    // Collect inputs from args and stdin
    let inputs = collect_inputs(&matches)?;
    if inputs.is_empty() {
        cmd.print_long_help()?;
        process::exit(1);
    }

    // Handle default excludes
    let mut exclude = GlobSetBuilder::new();
    if !matches.get_flag("no-default-excludes") {
        for spec in specs() {
            for pattern in spec.exclude {
                exclude.add(
                    GlobBuilder::new(pattern)
                        .literal_separator(false)
                        .build()
                        .unwrap(),
                );
            }
        }
    }

    // Handle CLI-provided excludes
    if let Some(pats) = matches.get_many::<String>("exclude") {
        for pattern in pats {
            exclude.add(
                GlobBuilder::new(pattern)
                    .literal_separator(false)
                    .build()
                    .unwrap(),
            );
        }
    }

    let excludes = exclude.build().unwrap();
    let defaults_enabled = !matches.get_flag("no-default-excludes");
    for path in inputs {
        let mut context = LangContext::new(&matches, &path, &excludes, defaults_enabled);
        match handle_path(&mut context) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("join: Error reading {path:?}: {e}");
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn handle_path(context: &mut LangContext) -> Result<(), std::io::Error> {
    if context.excluded() || !context.visit() {
        return Ok(());
    }
    let meta = fs::metadata(context.path)?;

    if meta.is_file() {
        let contents = fs::read_to_string(context.path);
        match contents {
            Err(e) => {
                eprintln!("Error {:?}: {e}", context.path);
                return Ok(());
            }
            Ok(contents) => {
                let spec = specs()
                    .into_iter()
                    .find(|spec| spec.matches.spec_matches(context))
                    .unwrap_or_default();
                let processed = spec.processor.process_contents(&context, contents);
                let formatted = spec.format.format_contents(&context, processed);
                print!("{}", formatted);
                return Ok(());
            }
        };
    }

    let entries = fs::read_dir(context.path)?;
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::metadata(&path)?;
        if context.excluded() {
            continue;
        }
        if meta.is_file() {
            files.push(path);
        } else if meta.is_dir() {
            dirs.push(path);
        }
    }

    for spec in specs() {
        spec.sort.sort_files(&mut files)?;
    }

    // Process files first, then directories
    for path in files {
        handle_path(&mut context.child(&path))?;
    }
    for path in dirs {
        handle_path(&mut context.child(&path))?;
    }

    Ok(())
}

fn collect_inputs(matches: &clap::ArgMatches) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut inputs: Vec<PathBuf> = matches
        .get_many::<String>("inputs")
        .map(|vals| vals.map(PathBuf::from).collect())
        .unwrap_or_default();

    // Read from stdin if piped
    if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                inputs.push(PathBuf::from(trimmed));
            }
        }
    }

    Ok(inputs)
}
