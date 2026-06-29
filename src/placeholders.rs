use crate::data_file_parser::Node;
use std::collections::HashMap;

pub mod categories;
pub mod pages;

/// Converts the given page yaml into a [`Node`] while embedding the virtual placeholders that are
/// shared across the blog: `PAGES` (see [`pages::build`]) and `CATEGORIES` (see
/// [`categories::build`]).
pub fn insert_virtual_placeholders<'a>(
    value: &'a serde_yaml::Value,
    pages: &Node<'a>,
    categories: &Node<'a>,
) -> Node<'a> {
    let mut root_map = match Node::from_yaml(value) {
        Node::Map(map) => map,
        _ => HashMap::new(),
    };
    root_map.insert("PAGES", pages.clone());
    root_map.insert("CATEGORIES", categories.clone());
    Node::Map(root_map)
}
