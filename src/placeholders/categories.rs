use crate::data_file_parser::Node;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

/// Intermediate node used to incrementally build the CATEGORIES tree before converting it into a
/// [`Node`].
#[derive(Default)]
struct Category<'a> {
    pages: Vec<Rc<Node>>,
    subcategories: BTreeMap<&'a str, Category<'a>>,
}

/// Builds a category forest by grouping individual data set trees into (sub)categories based on
/// their category paths as declared in each data set tree under the `categories` key.
///
/// The resulting category forest is returned as a [`Node::Seq`], where each node represents
/// a (sub)category and is a [`Node::Map`], exposing `name`, `pages` and `subcategories`.
///
/// Pages without a `categories` chain are not included in the tree.
pub fn build(data_set_trees: &[Rc<Node>]) -> Rc<Node> {
    let mut categories = BTreeMap::new();
    for data_set_tree in data_set_trees {
        if let Some(category_path) = get_category_path(data_set_tree.as_ref()) {
            insert_data_set_tree(&mut categories, &category_path, data_set_tree);
        }
    }
    to_tree(categories)
}

/// Extracts the `categories` chain from the given data set tree, if it is present and non-empty.
fn get_category_path(data_set_tree: &Node) -> Option<Vec<&str>> {
    let categories = match data_set_tree {
        Node::Map(map) => map.get("categories")?,
        _ => return None,
    };
    let categories = match categories.as_ref() {
        Node::Seq(seq) => seq,
        _ => return None,
    };
    let path: Vec<&str> = categories
        .iter()
        .filter_map(|segment| match segment.as_ref() {
            Node::Str(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    if path.is_empty() { None } else { Some(path) }
}

/// Inserts the given data set tree into the categories forest according to its category path.
fn insert_data_set_tree<'a>(
    categories: &mut BTreeMap<&'a str, Category<'a>>,
    category_path: &[&'a str],
    tree: &Rc<Node>,
) {
    let category = categories.entry(category_path[0]).or_default();
    let rest = &category_path[1..];
    if rest.is_empty() {
        category.pages.push(Rc::clone(tree));
    } else {
        insert_data_set_tree(&mut category.subcategories, rest, tree);
    }
}

/// Converts a category tree into a [`Node::Seq`] representation.
fn to_tree(categories: BTreeMap<&str, Category<'_>>) -> Rc<Node> {
    Rc::new(Node::Seq(
        categories
            .into_iter()
            .map(|(name, category)| to_node(name, category))
            .collect(),
    ))
}

/// Converts a category into a [`Node::Map`] representation.
fn to_node(name: &str, category: Category<'_>) -> Rc<Node> {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Rc::new(Node::Str(name.to_string())));
    map.insert("pages".to_string(), Rc::new(Node::Seq(category.pages)));
    map.insert("subcategories".to_string(), to_tree(category.subcategories));
    Rc::new(Node::Map(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_yaml_texts(yamls: &[&str]) -> Result<Vec<Rc<Node>>, String> {
        yamls
            .iter()
            .map(|yaml| Node::from_yaml_text(yaml))
            .collect()
    }

    fn child<'a>(node: &'a Node, key: &str) -> &'a Node {
        match node {
            Node::Map(map) => map
                .get(key)
                .map(Rc::as_ref)
                .unwrap_or_else(|| panic!("missing key '{key}'")),
            other => panic!("expected a map, got {other:?}"),
        }
    }

    fn seq(node: &Node) -> &[Rc<Node>] {
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
    fn build_nests_a_page_under_its_full_category_path() {
        let nodes =
            parse_yaml_texts(&["title: Oats\ncategories: [home, cooking, recipes]"]).unwrap();

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
    fn build_ignores_pages_without_a_categories_path() {
        let nodes = parse_yaml_texts(&[
            "title: Post\ncategories: [home, blog]",
            "title: Standalone",
            "title: Empty\ncategories: []",
            "title: Scalar\ncategories: home",
        ])
        .unwrap();

        let categories = build(&nodes);

        assert_eq!(category_names(&categories), vec!["home"]);
        let home = &seq(&categories)[0];
        assert_eq!(category_names(child(home, "subcategories")), vec!["blog"]);
        let blog = &seq(child(home, "subcategories"))[0];
        assert_eq!(page_titles(blog), vec!["Post"]);
    }

    #[test]
    fn build_lets_a_category_hold_both_pages_and_subcategories() {
        let nodes = parse_yaml_texts(&[
            "title: Finance\ncategories: [home, finance]",
            "title: Car Clowns\ncategories: [home, finance, mmm]",
        ])
        .unwrap();

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
