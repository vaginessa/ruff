use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use ::rust_python_linter::fs::collect_python_files;
use ::rust_python_linter::logging::set_up_logging;
use ::rust_python_linter::tell_user;
use ::rust_python_linter::test_runner::check_path;
use anyhow::Result;
use clap::{Parser, ValueHint};
use colored::Colorize;
use log::{debug, error};
use notify::{watcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use rust_python_linter::test_runner::Module;

#[derive(Debug, Parser)]
#[clap(name = "rust-python-linter")]
#[clap(about = "A bare-bones Python linter written in Rust", long_about = None)]
struct Cli {
    #[clap(parse(from_os_str), value_hint = ValueHint::DirPath, required = true)]
    files: Vec<PathBuf>,
    #[clap(short, long, action)]
    verbose: bool,
    #[clap(short, long, action)]
    watch: bool,
    #[clap(short, long, action)]
    no_cache: bool,
}

fn run_once(files: &[PathBuf], cache: bool) -> Result<()> {
    // Collect all the files to check.
    let start = Instant::now();
    let modules: Vec<Module> = files
        .iter()
        .flat_map(|source_root| {
            collect_python_files(source_root)
                .iter()
                .map(|path| Module {
                    source_root: source_root.clone(),
                    path: path.path().to_path_buf(),
                })
                .collect::<Vec<Module>>()
        })
        .collect();
    let duration = start.elapsed();
    debug!("Identified files to test in: {:?}", duration);

    let start = Instant::now();
    let _: Vec<()> = modules
        .par_iter()
        .map(|module| {
            check_path(module, &cache.into()).unwrap_or_else(|e| {
                error!("Failed to test {}: {e:?}", module.path.to_string_lossy());
            })
        })
        .collect();
    let duration = start.elapsed();
    debug!("Ran tests in: {:?}", duration);

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    set_up_logging(cli.verbose)?;

    if cli.watch {
        // Perform an initial run instantly.
        clearscreen::clear()?;
        tell_user!("Starting runner in watch mode...\n");

        run_once(&cli.files, !cli.no_cache)?;

        // Configure the file watcher.
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(2))?;
        for file in &cli.files {
            watcher.watch(file, RecursiveMode::Recursive)?;
        }

        loop {
            match rx.recv() {
                Ok(_) => {
                    // Re-run on all change events.
                    clearscreen::clear()?;
                    tell_user!("File change detected...\n");

                    run_once(&cli.files, !cli.no_cache)?;
                }
                Err(e) => return Err(e.into()),
            }
        }
    } else {
        run_once(&cli.files, !cli.no_cache)?;
    }

    Ok(())
}
