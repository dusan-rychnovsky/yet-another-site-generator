use crate::data_file_parser::Node;

/// Builds the `PAGES` placeholder: a [`Node::Seq`] holding the datasource of every page in the blog.
pub fn build<'a>(page_nodes: &[Node<'a>]) -> Node<'a> {
    Node::Seq(page_nodes.to_vec())
}
