use anyhow::anyhow;
use clap::{Parser, ValueHint};
use ignore::WalkBuilder;
use kitokei::{combine_counts, parse_file};
use std::path::PathBuf;
use tokio::task::JoinSet;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.path.exists() {
        return Err(anyhow::anyhow!("Path does not exist"));
    }

    match args.path {
        path if args.path.is_file() => {
            let counts = parse_file(&path).await?;
            let table = kitokei::table(counts);
            println!("{}", table);
        }
        path if args.path.is_dir() => {
            let mut set = JoinSet::new();
            // TODO: look into WalkBuilder::build_parallel
            WalkBuilder::new(path).build().for_each(|e| match e {
                Ok(v) if v.path().is_file() => {
                    set.spawn(async move { kitokei::parse_file(v.path()).await });
                }
                Err(error) => {
                    println!("{}", error);
                }
                _ => {}
            });
            let mut all_counts = Vec::new();
            while let Some(result) = set.join_next().await {
                match result {
                    Ok(Ok(counts)) => {
                        all_counts.push(counts);
                    }
                    Ok(Err(error)) => {
                        if args.skipped {
                            println!("Skipped: {}", error);
                        }
                    }
                    Err(error) => {
                        set.abort_all();
                        anyhow::bail!(error);
                    }
                }
            }
            let table = kitokei::table(
                all_counts
                    .into_iter()
                    .reduce(combine_counts)
                    .ok_or_else(|| anyhow!("No files to analyze"))?,
            );
            println!("{}", table);
        }
        _ => return Err(anyhow::anyhow!("Path is not a file or directory")),
    }
    Ok(())
}
