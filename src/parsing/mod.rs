#![allow(unused, dead_code)]

use std::{collections::BTreeMap, path::Path};

use eyre::{bail, ContextCompat};
use itertools::Itertools;
use lazy_static::lazy_static;
use tree_sitter::{InputEdit, Language, Parser, Point, Query, QueryCursor};

// NOTE: can use `LazyCell` on `Rust` >= 1.80.0, but the `time` crate doesn't compile there

lazy_static! {
    pub static ref LANGUAGE: Language = tree_sitter_rust::language();
}

pub static FUNCTION_QUERY_STR: &str = /* query */
    r#"
(function_item
  name: (identifier) @name)
"#;

lazy_static! {
    pub static ref FUNCTION_QUERY: Query =
        Query::new(&LANGUAGE, FUNCTION_QUERY_STR).unwrap();
}

pub static FUNCTION_IN_PROGRAM_MODULE_QUERY_STR: &str = /* query */
    r#"
(
  (
    (attribute_item
      (attribute
        (identifier) @_program))
    (mod_item
      body:
      (declaration_list
        (function_item
           name: (identifier) @name)))
  )
  (#match? @_program "^program$")
)
"#;

lazy_static! {
    pub static ref FUNCTION_IN_PROGRAM_MODULE_QUERY: Query =
        Query::new(&LANGUAGE, FUNCTION_IN_PROGRAM_MODULE_QUERY_STR).unwrap();
}

pub fn extract_functions(
    file_path: impl AsRef<Path>,
    query: &Query,
) -> eyre::Result<BTreeMap<String, (Point, Point)>> {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE)?;

    let source_code = std::fs::read_to_string(file_path)?;
    let tree = parser.parse(source_code.as_bytes(), None).unwrap();
    let root_node = tree.root_node();

    let mut query_cursor = QueryCursor::new();
    let matches =
        query_cursor.matches(query, root_node, source_code.as_bytes());

    let results = matches
        .map(|m| -> eyre::Result<_> {
            let (name, node) = m
                .captures
                .iter()
                .find_map(|capture| {
                    query
                        .capture_names()
                        .get(capture.index as usize)
                        .filter(|name| **name == "name")
                        .map(|name| (name, capture.node))
                })
                .wrap_err("Could not find `name` capture")?;

            let start = node.start_position();
            let end = node.end_position();
            let text = node.utf8_text(source_code.as_bytes()).unwrap();

            Ok((text.to_string(), (start, end)))
        })
        .try_collect()?;

    Ok(results)
}
