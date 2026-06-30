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

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the keys of the given map node, sorted for stable comparison.
    fn keys<'a>(node: &'a Node<'a>) -> Vec<&'a str> {
        match node {
            Node::Map(map) => {
                let mut keys: Vec<&str> = map.keys().copied().collect();
                keys.sort();
                keys
            }
            other => panic!("expected a map, got {other:?}"),
        }
    }

    #[test]
    fn insert_virtual_placeholders_adds_placeholders_and_keeps_page_fields() {
        let value: serde_yaml::Value = serde_yaml::from_str("title: Hello\nauthor: Jane").unwrap();
        let pages = Node::Seq(Vec::new());
        let categories = Node::Seq(Vec::new());

        let root = insert_virtual_placeholders(&value, &pages, &categories);

        assert_eq!(keys(&root), vec!["CATEGORIES", "PAGES", "author", "title"]);
    }
}
