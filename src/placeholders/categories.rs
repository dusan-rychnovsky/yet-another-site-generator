use crate::data_file_parser::Node;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// Intermediate node used to incrementally build the CATEGORIES tree before converting it into a
/// [`Node`].
#[derive(Default)]
struct CategoryBuilder<'a> {
    pages: Vec<Node<'a>>,
    subcategories: BTreeMap<&'a str, CategoryBuilder<'a>>,
}

/// Builds the `CATEGORIES` placeholder by grouping the given pages into a tree of categories based
/// on the `categories` chain declared in each page. Returns the tree as a [`Node::Seq`] of category
/// nodes, where each category node is a [`Node::Map`] exposing `name`, `pages` and `subcategories`.
/// Pages without a `categories` chain are not included in the tree.
pub fn build<'a>(page_nodes: &[Node<'a>]) -> Node<'a> {
    let mut roots: BTreeMap<&'a str, CategoryBuilder<'a>> = BTreeMap::new();
    for page in page_nodes {
        if let Some(chain) = get_category_chain(page) {
            insert_page(&mut roots, &chain, page);
        }
    }
    categories_to_node(roots)
}

/// Extracts the `categories` chain from the given page node, if it is present and non-empty.
fn get_category_chain<'a>(page: &Node<'a>) -> Option<Vec<&'a str>> {
    let categories = match page {
        Node::Map(map) => map.get("categories")?,
        _ => return None,
    };
    let segments = match categories {
        Node::Seq(seq) => seq,
        _ => return None,
    };
    let chain: Vec<&'a str> = segments
        .iter()
        .filter_map(|segment| match segment {
            Node::Str(Cow::Borrowed(name)) => Some(*name),
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
    page: &Node<'a>,
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
fn categories_to_node<'a>(categories: BTreeMap<&'a str, CategoryBuilder<'a>>) -> Node<'a> {
    Node::Seq(
        categories
            .into_iter()
            .map(|(name, category)| category_to_node(name, category))
            .collect(),
    )
}

/// Converts a single named [`CategoryBuilder`] into a [`Node::Map`] exposing `name`, `pages` and
/// `subcategories`.
fn category_to_node<'a>(name: &'a str, category: CategoryBuilder<'a>) -> Node<'a> {
    let mut map = HashMap::new();
    map.insert("name", Node::Str(Cow::Borrowed(name)));
    map.insert("pages", Node::Seq(category.pages));
    map.insert("subcategories", categories_to_node(category.subcategories));
    Node::Map(map)
}
