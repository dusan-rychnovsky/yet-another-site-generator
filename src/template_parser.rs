use crate::template_tokenizer::{self, TemplateToken};
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

fn parse_tree(input: &str) -> Result<TemplateTree, String> {
  let tokens = template_tokenizer::tokenize_content(input)
    .map_err(|e| format!("Failed to tokenize template: {}", e))?;

  let mut seq: Vec<Box<TemplateNode>> = Vec::new();
  for token in tokens {
    match token {
      TemplateToken::Text(text) => {
        seq.push(Box::new(TemplateNode::Text(text)));
      },
      other => {
        return Err(format!("Unexpected tokken: {:?}", other));
      }
    }
  }
  Ok(
    TemplateTree {
      root: TemplateNode::Seq(seq)
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

  #[test]
  fn parse_tree_handles_simple_text() {
    let input = "This is a simple text.";
    let result = parse_tree(input).unwrap();
    assert_eq!(
      result.root,
      TemplateNode::Seq(
        vec![
          Box::new(TemplateNode::Text(input.to_string()))
        ]
      )
    );
  }
}