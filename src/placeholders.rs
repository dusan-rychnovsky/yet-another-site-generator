use crate::data_file_parser::Node;
use std::collections::HashMap;

pub mod categories;
pub mod pages;
pub mod path;

/// Returns a copy of the given page node with the root-level virtual placeholders inserted:
/// `PAGES` (see [`pages::build`]) and `CATEGORIES` (see [`categories::build`]).
pub fn insert_virtual_placeholders(page: &Node, pages: &Node, categories: &Node) -> Node {
    let mut root_map = match page {
        Node::Map(map) => map.clone(),
        _ => HashMap::new(),
    };
    root_map.insert("PAGES".to_string(), pages.clone());
    root_map.insert("CATEGORIES".to_string(), categories.clone());
    Node::Map(root_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the keys of the given map node, sorted for stable comparison.
    fn keys(node: &Node) -> Vec<&str> {
        match node {
            Node::Map(map) => {
                let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
                keys.sort();
                keys
            }
            other => panic!("expected a map, got {other:?}"),
        }
    }

    #[test]
    fn insert_virtual_placeholders_adds_placeholders_and_keeps_page_fields() {
        let value: serde_yaml::Value = serde_yaml::from_str("title: Hello\nauthor: Jane").unwrap();
        let page = Node::from_yaml(&value);
        let pages = Node::Seq(Vec::new());
        let categories = Node::Seq(Vec::new());

        let root = insert_virtual_placeholders(&page, &pages, &categories);

        assert_eq!(keys(&root), vec!["CATEGORIES", "PAGES", "author", "title"]);
    }
}
