use anyhow::anyhow;
use clap::{Parser, ValueHint};
use kitokei::capture_counts;
use kitokei::language::Language;
use std::collections::HashMap;
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the file or directory to analyze
    #[arg(value_name = "PATH", value_hint = ValueHint::AnyPath)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.path.exists() {
        return Err(anyhow::anyhow!("Path does not exist"));
    }

    let mut parser = tree_sitter::Parser::new();
    match args.path {
        path if args.path.is_file() => {
            let lang = Language::try_from(path.as_path())?;
            let ts_lang = lang.tree_sitter_language()?;
            parser.set_language(ts_lang)?;
            let query = tree_sitter::Query::new(ts_lang, lang.queries()?)?;
            let code = std::fs::read_to_string(&path)?;
            let tree = parser
                .parse(&code, None)
                .ok_or_else(|| anyhow!("Failed to parse file"))?;

            let capture_names = query.capture_names();
            let counts = capture_counts(&query, tree.root_node(), &code)
                .into_iter()
                .map(|(k, v)| (capture_names[k].as_str(), v))
                .collect::<HashMap<_, _>>();
            let table = kitokei::table(counts);
            println!("{}", table);
        }
        _ => {
            unimplemented!("Directory analysis not implemented");
        }
    }
    Ok(())
}
