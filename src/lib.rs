pub mod language;

use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use itertools::Itertools;
use std::collections::HashMap;
use tree_sitter::QueryCursor;

/// Count the number of times each capture group was matched by `query`
pub fn capture_counts(
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

/// Combine the counts of two maps
pub fn combine_counts(
    mut counts: HashMap<usize, usize>,
    new_counts: HashMap<usize, usize>,
) -> HashMap<usize, usize> {
    for (k, v) in new_counts {
        *counts.entry(k).or_insert(0) += v;
    }
    counts
}

/// Pretty print a table of counts
pub fn table(counts: HashMap<&str, usize>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["Query", "Count"])
        .add_rows(
            counts
                .into_iter()
                .sorted()
                .filter_map(|(item, count)| match item {
                    _ if item.starts_with('_') => None,
                    _ => Some([item.to_string(), count.to_string()]),
                }),
        );
    table
}
