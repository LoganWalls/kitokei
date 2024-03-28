use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use tree_sitter::{Parser, QueryCursor};

use arrow_array::array::StringArray;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

fn capture_counts(
    query: &tree_sitter::Query,
    root: tree_sitter::Node,
    code: &str,
) -> HashMap<usize, usize> {
    let src = code.as_bytes();
    let mut cursor = QueryCursor::new();
    cursor
        .matches(query, root, src)
        .flat_map(|m| m.captures)
        .map(|c| c.index as usize)
        .counts()
}

fn combine_counts(
    mut counts: HashMap<usize, usize>,
    new_counts: HashMap<usize, usize>,
) -> HashMap<usize, usize> {
    for (k, v) in new_counts {
        *counts.entry(k).or_insert(0) += v;
    }
    counts
}

fn print_table(counts: HashMap<&str, usize>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["Query", "Count"])
        .add_rows(
            counts
                .into_iter()
                .sorted()
                .map(|(item, count)| vec![item.to_string(), count.to_string()]),
        );
    println!("{table}");
}

fn main() -> anyhow::Result<()> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::language();
    parser.set_language(&language)?;
    let query = tree_sitter::Query::new(&language, include_str!("../data/python-highlights.scm"))?;
    let capture_names = query.capture_names();

    let file = File::open("../data/python.parquet").unwrap();
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();
    let reader = builder.build().unwrap();
    let counts = reader
        .into_iter()
        .flatten()
        .map(|batch| {
            // Get the column with the source code
            let values: &StringArray = batch
                .column_by_name("content")
                .expect("'content' column exists")
                .as_any()
                .downcast_ref()
                .expect("column is a string array");
            // Map the source code to an iterator of capture names
            values
                .into_iter()
                .flatten()
                .map(|code| {
                    let tree = parser.parse(code, None).expect("can parse code");
                    capture_counts(&query, tree.root_node(), code)
                })
                .reduce(combine_counts)
                .unwrap_or(HashMap::new())
        })
        .reduce(combine_counts)
        .unwrap_or(HashMap::new())
        .into_iter()
        .map(|(k, v)| (capture_names[k], v))
        .collect::<HashMap<_, _>>();

    print_table(counts);
    Ok(())
}
