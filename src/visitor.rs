use crate::template_parser::{TemplateTree, TemplateNode, TemplateNode::*};

pub fn visit(tree: &TemplateTree) -> String {
  let mut output = String::new();
  visit_node(&tree.root, &mut output);
  output
}

fn visit_node(node: &TemplateNode, output: &mut String) {
  match node {
    Text(text) => {
      output.push_str(text);
    },
    _ => ()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn visit_simple_text() {
    let tree = TemplateTree {
      root: TemplateNode::Text("Hello, world!"),
    };
    let result = visit(&tree);
    assert_eq!(result, "Hello, world!");
  }
}
