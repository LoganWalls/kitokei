use clap::{Parser, ValueHint};
use ignore::WalkBuilder;
use kitokei::{combine_counts, parse_file};
use std::path::PathBuf;

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

    match args.path {
        path if args.path.is_file() => {
            let counts = parse_file(&path)?;
            let table = kitokei::table(counts);
            println!("{}", table);
        }
        path if args.path.is_dir() => {
            // TODO: look into WalkBuilder::build_parallel
            let counts = WalkBuilder::new(path)
                .build()
                .filter_map(|e| match e {
                    Ok(v) if v.path().is_file() => match kitokei::parse_file(v.path()) {
                        Ok(counts) => Some(counts),
                        Err(error) => {
                            if args.skipped {
                                println!("Skipped: {}", error);
                            }
                            None
                        }
                    },
                    Err(error) => {
                        println!("{}", error);
                        None
                    }
                    _ => None,
                })
                .reduce(combine_counts)
                .ok_or_else(|| anyhow::anyhow!("No files to analyze"))?;

            let table = kitokei::table(counts);
            println!("{}", table);
        }
        _ => return Err(anyhow::anyhow!("Path is not a file or directory")),
    }
    Ok(())
}
