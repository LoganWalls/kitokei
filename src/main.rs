use clap::{Parser, ValueHint};
use dashmap::DashMap;
use ignore::WalkBuilder;
use kitokei::{combine_counts, parse_file};
use rayon::prelude::*;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::available_parallelism;

/// Parse and query a file or directory with tree-sitter and report the number of times each query
/// is matched
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the file or directory to analyze
    #[arg(
        value_name = "PATH",
        value_hint = ValueHint::AnyPath,
        default_value = PathBuf::from(".").into_os_string())]
    path: PathBuf,
    /// Show skipped files
    #[arg(short, long)]
    skipped: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.path.exists() {
        return Err(anyhow::anyhow!("Path does not exist"));
    }
    let query_cache = DashMap::new();
    match args.path {
        path if args.path.is_file() => {
            let counts = parse_file(&path, &query_cache)?;
            let table = kitokei::table(counts);
            println!("{}", table);
        }
        path if args.path.is_dir() => {
            let (path_tx, path_rx) = mpsc::sync_channel(
                available_parallelism()
                    .unwrap_or(NonZeroUsize::new(2).unwrap())
                    .get(),
            );
            std::thread::spawn(move || {
                WalkBuilder::new(path).build_parallel().run(|| {
                    let tx = path_tx.clone();
                    Box::new(move |e| {
                        match e {
                            Ok(v) if v.path().is_file() => {
                                tx.send(v).expect("channel should be available to send");
                            }
                            Err(error) => {
                                println!("{}", error);
                            }
                            _ => {}
                        }
                        ignore::WalkState::Continue
                    })
                });
            });

            let counts = path_rx
                .into_iter()
                .par_bridge()
                .filter_map(
                    |entry| match kitokei::parse_file(entry.path(), &query_cache) {
                        Ok(counts) => Some(counts),
                        Err(error) => {
                            if args.skipped {
                                println!("Skipped: {}", error);
                            }
                            None
                        }
                    },
                )
                .reduce(HashMap::new, combine_counts);

            let table = kitokei::table(counts);
            println!("{}", table);
        }
        _ => return Err(anyhow::anyhow!("Path is not a file or directory")),
    }
    Ok(())
}
