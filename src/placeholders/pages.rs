use crate::data_file_parser::Node;

/// Builds the `PAGES` placeholder: a [`Node::Seq`] holding the datasource of every page in the blog.
pub fn build<'a>(page_nodes: &[Node<'a>]) -> Node<'a> {
    Node::Seq(page_nodes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_wraps_all_pages_in_a_sequence() {
        let values: Vec<serde_yaml::Value> = ["title: A", "title: B"]
            .iter()
            .map(|yaml| serde_yaml::from_str(yaml).unwrap())
            .collect();
        let nodes: Vec<Node> = values.iter().map(Node::from_yaml).collect();

        let pages = build(&nodes);

        match pages {
            Node::Seq(items) => assert_eq!(items.len(), 2),
            other => panic!("expected a sequence, got {other:?}"),
        }
    }
}
