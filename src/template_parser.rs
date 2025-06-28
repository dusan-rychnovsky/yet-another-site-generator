use std::fs;

pub struct TemplateTree {
  pub root: TemplateNode
}

#[derive(Debug, PartialEq)]
pub enum TemplateNode {
  Seq(Vec<Box<TemplateNode>>),
  Text(String), // TODO: replace with string slice with lifetime
  Var(String),
  ForEach (String, String, Box<TemplateNode>),
  If (String, Box<TemplateNode>)
}

pub fn parse(path: &str) -> Result<TemplateTree, Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;
  let tree = parse_tree(&content)?;
  Ok(tree)
}

/*
// This is a stub. You'd want to implement a real parser here.
fn parse_nodes(input: &str) -> Result<Vec<Box<TemplateNode>>, Box<dyn Error>> {
  let mut nodes = Vec::new();
  let mut rest = input;

  while !rest.is_empty() {
    if let Some(start) = rest.find("{{") {
      // Text before variable
      if start > 0 {
        nodes.push(Box::new(TemplateNode::Text(rest[..start].to_string())));
      }
      if let Some(end) = rest.find("}}") {
        let var = rest[start + 2..end].trim().to_string();
        nodes.push(Box::new(TemplateNode::Var(var)));
        rest = &rest[end + 2..];
      } else {
        break; // Malformed template
      }
    } else {
      // All remaining is text
      nodes.push(Box::new(TemplateNode::Text(rest.to_string())));
      break;
    }
  }

  Ok(nodes)
}
*/

fn parse_tree(input: &str) -> Result<TemplateTree, Box<dyn std::error::Error>> {
  Ok(
    TemplateTree {
      root: TemplateNode::Seq(Vec::new())
    }
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_tree_handles_empty_input() {
    let result = parse_tree("").unwrap();
    assert_eq!(result.root, TemplateNode::Seq(Vec::new()));
  }
}