use crate::data_file_parser::Node;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// Intermediate node used to incrementally build the CATEGORIES tree before converting it into a
/// [`Node`].
#[derive(Default)]
struct CategoryBuilder<'a> {
    pages: Vec<Node>,
    subcategories: BTreeMap<&'a str, CategoryBuilder<'a>>,
}

/// Builds the `CATEGORIES` placeholder by grouping the given pages into a tree of categories based
/// on the `categories` chain declared in each page. Returns the tree as a [`Node::Seq`] of category
/// nodes, where each category node is a [`Node::Map`] exposing `name`, `pages` and `subcategories`.
/// Pages without a `categories` chain are not included in the tree.
pub fn build(page_nodes: &[Node]) -> Node {
    let mut roots: BTreeMap<&str, CategoryBuilder> = BTreeMap::new();
    for page in page_nodes {
        if let Some(chain) = get_category_chain(page) {
            insert_page(&mut roots, &chain, page);
        }
    }
    categories_to_node(roots)
}

/// Extracts the `categories` chain from the given page node, if it is present and non-empty.
fn get_category_chain(page: &Node) -> Option<Vec<&str>> {
    let categories = match page {
        Node::Map(map) => map.get("categories")?,
        _ => return None,
    };
    let segments = match categories {
        Node::Seq(seq) => seq,
        _ => return None,
    };
    let chain: Vec<&str> = segments
        .iter()
        .filter_map(|segment| match segment {
            Node::Str(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    if chain.is_empty() { None } else { Some(chain) }
}

/// Inserts the given page into the categories tree, following (and creating as needed) the
/// categories named by `chain`. The page is assigned to the last category in the chain.
fn insert_page<'a>(
    categories: &mut BTreeMap<&'a str, CategoryBuilder<'a>>,
    chain: &[&'a str],
    page: &Node,
) {
    let category = categories.entry(chain[0]).or_default();
    let rest = &chain[1..];
    if rest.is_empty() {
        category.pages.push(page.clone());
    } else {
        insert_page(&mut category.subcategories, rest, page);
    }
}

/// Converts a map of named [`CategoryBuilder`]s into a [`Node::Seq`] of category nodes.
fn categories_to_node(categories: BTreeMap<&str, CategoryBuilder<'_>>) -> Node {
    Node::Seq(
        categories
            .into_iter()
            .map(|(name, category)| category_to_node(name, category))
            .collect(),
    )
}

/// Converts a single named [`CategoryBuilder`] into a [`Node::Map`] exposing `name`, `pages` and
/// `subcategories`.
fn category_to_node(name: &str, category: CategoryBuilder<'_>) -> Node {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Node::Str(name.to_string()));
    map.insert("pages".to_string(), Node::Seq(category.pages));
    map.insert(
        "subcategories".to_string(),
        categories_to_node(category.subcategories),
    );
    Node::Map(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses the given yaml documents into values, keeping them alive so [`Node`]s can borrow them.
    fn parse_pages(yamls: &[&str]) -> Vec<serde_yaml::Value> {
        yamls
            .iter()
            .map(|yaml| serde_yaml::from_str(yaml).expect("valid yaml"))
            .collect()
    }

    fn child<'a>(node: &'a Node, key: &str) -> &'a Node {
        match node {
            Node::Map(map) => map
                .get(key)
                .unwrap_or_else(|| panic!("missing key '{key}'")),
            other => panic!("expected a map, got {other:?}"),
        }
    }

    fn seq(node: &Node) -> &[Node] {
        match node {
            Node::Seq(items) => items,
            other => panic!("expected a sequence, got {other:?}"),
        }
    }

    fn text(node: &Node) -> &str {
        match node {
            Node::Str(value) => value.as_str(),
            other => panic!("expected a string, got {other:?}"),
        }
    }

    /// Names of the categories in the given `CATEGORIES`/`subcategories` sequence, in order.
    fn category_names(categories: &Node) -> Vec<&str> {
        seq(categories)
            .iter()
            .map(|category| text(child(category, "name")))
            .collect()
    }

    /// Titles of the pages directly assigned to the given category, in order.
    fn page_titles(category: &Node) -> Vec<&str> {
        seq(child(category, "pages"))
            .iter()
            .map(|page| text(child(page, "title")))
            .collect()
    }

    #[test]
    fn build_nests_a_page_under_its_full_category_chain() {
        let values = parse_pages(&["title: Oats\ncategories: [home, cooking, recipes]"]);
        let nodes: Vec<Node> = values.iter().map(Node::from_yaml).collect();

        let categories = build(&nodes);

        assert_eq!(category_names(&categories), vec!["home"]);
        let home = &seq(&categories)[0];
        assert!(page_titles(home).is_empty());

        assert_eq!(
            category_names(child(home, "subcategories")),
            vec!["cooking"]
        );
        let cooking = &seq(child(home, "subcategories"))[0];
        assert!(page_titles(cooking).is_empty());

        assert_eq!(
            category_names(child(cooking, "subcategories")),
            vec!["recipes"]
        );
        let recipes = &seq(child(cooking, "subcategories"))[0];
        assert_eq!(page_titles(recipes), vec!["Oats"]);
        assert!(category_names(child(recipes, "subcategories")).is_empty());
    }

    #[test]
    fn build_ignores_pages_without_a_categories_chain() {
        let values = parse_pages(&[
            "title: Post\ncategories: [home, blog]",
            "title: Standalone",
            "title: Empty\ncategories: []",
            "title: Scalar\ncategories: home",
        ]);
        let nodes: Vec<Node> = values.iter().map(Node::from_yaml).collect();

        let categories = build(&nodes);

        assert_eq!(category_names(&categories), vec!["home"]);
        let home = &seq(&categories)[0];
        assert_eq!(category_names(child(home, "subcategories")), vec!["blog"]);
        let blog = &seq(child(home, "subcategories"))[0];
        assert_eq!(page_titles(blog), vec!["Post"]);
    }

    #[test]
    fn build_lets_a_category_hold_both_pages_and_subcategories() {
        let values = parse_pages(&[
            "title: Finance\ncategories: [home, finance]",
            "title: Car Clowns\ncategories: [home, finance, mmm]",
        ]);
        let nodes: Vec<Node> = values.iter().map(Node::from_yaml).collect();

        let categories = build(&nodes);

        let home = &seq(&categories)[0];
        assert_eq!(
            category_names(child(home, "subcategories")),
            vec!["finance"]
        );
        let finance = &seq(child(home, "subcategories"))[0];
        assert_eq!(page_titles(finance), vec!["Finance"]);
        assert_eq!(category_names(child(finance, "subcategories")), vec!["mmm"]);
        let mmm = &seq(child(finance, "subcategories"))[0];
        assert_eq!(page_titles(mmm), vec!["Car Clowns"]);
    }
}
