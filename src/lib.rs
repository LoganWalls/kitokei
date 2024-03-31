pub mod language;

use anyhow::{anyhow, Result};
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::QueryCursor;

use self::language::Language;

/// Parse a file and return the counts of each capture group
pub fn parse_file(
    path: &Path,
    queries: &DashMap<Language, tree_sitter::Query>,
) -> Result<HashMap<String, usize>> {
    let mut parser = tree_sitter::Parser::new();
    let lang = Language::try_from(path)?;
    let ts_lang = lang.tree_sitter_language()?;
    parser.set_language(ts_lang)?;
    let query =
        queries
            .entry(lang)
            .or_try_insert_with(|| -> Result<tree_sitter::Query> {
                Ok(tree_sitter::Query::new(ts_lang, lang.queries()?)?)
            })?;
    let code = std::fs::read_to_string(path)?;
    let tree = parser
        .parse(&code, None)
        .ok_or_else(|| anyhow!("Failed to parse file"))?;
    let capture_names = query.capture_names();
    Ok(capture_counts(&query, tree.root_node(), &code)
        .into_iter()
        .map(|(k, v)| (capture_names[k].to_string(), v))
        .collect::<HashMap<_, _>>())
}

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
pub fn combine_counts<K: std::hash::Hash + std::cmp::Eq>(
    mut counts: HashMap<K, usize>,
    new_counts: HashMap<K, usize>,
) -> HashMap<K, usize> {
    for (k, v) in new_counts {
        *counts.entry(k).or_insert(0) += v;
    }
    counts
}

/// Pretty print a table of counts
pub fn table<T: ToString + std::cmp::Ord>(counts: HashMap<T, usize>) -> Table {
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
                .filter_map(|(item, count)| match item.to_string() {
                    s if s.starts_with('_') => None,
                    s => Some([s, count.to_string()]),
                }),
        );
    table
}
