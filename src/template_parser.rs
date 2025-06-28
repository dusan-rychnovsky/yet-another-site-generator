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
  let (nodes, _) = parse_nodes(&tokens, 0)?;
  Ok(TemplateTree {
    root: TemplateNode::Seq(nodes),
  })
}

fn parse_nodes(tokens: &[TemplateToken], start_pos: usize) -> Result<(Vec<Box<TemplateNode>>, usize), String> {
  let mut nodes = Vec::new();
  let mut pos = start_pos;
  while pos < tokens.len() {
    let token = &tokens[pos];
    pos += 1;
    match token {
      TemplateToken::Text(text) => {
        nodes.push(Box::new(TemplateNode::Text(text.clone())));
      }
      TemplateToken::Var(var) => {
        nodes.push(Box::new(TemplateNode::Var(var.clone())));
      }
      TemplateToken::For(var, expr) => {
        let (body, new_start_pos) = parse_nodes(tokens, pos)?;
        nodes.push(Box::new(TemplateNode::ForEach(
          var.clone(),
          expr.clone(),
          Box::new(TemplateNode::Seq(body))
        )));
        pos = new_start_pos;
      }
      TemplateToken::EndFor(_) => {
        break;
      }
      other => {
        return Err(format!("Unexpected token: {:?}", other));
      }
    }
  }
  Ok((nodes, pos))
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

  #[test]
  fn parse_tree_handles_text_with_variables() {
    let input = "Hello, [name]! Welcome to [place.address].";
    let result = parse_tree(input).unwrap();
    assert_eq!(
      result.root,
      TemplateNode::Seq(
        vec![
          Box::new(TemplateNode::Text("Hello, ".to_string())),
          Box::new(TemplateNode::Var("name".to_string())),
          Box::new(TemplateNode::Text("! Welcome to ".to_string())),
          Box::new(TemplateNode::Var("place.address".to_string())),
          Box::new(TemplateNode::Text(".".to_string()))
        ]
      )
    );
  }

  #[test]
  fn parse_tree_handles_foreach() {
    let input = "[for section in sections] Section. Title: [section.title][endfor section]";
    let result = parse_tree(input).unwrap();
    assert_eq!(
      result.root,
      TemplateNode::Seq(
        vec![
          Box::new(TemplateNode::ForEach(
            "section".to_string(),
            "sections".to_string(),
            Box::new(TemplateNode::Seq(vec![
              Box::new(TemplateNode::Text(" Section. Title: ".to_string())),
              Box::new(TemplateNode::Var("section.title".to_string()))
            ]))
          ))
        ]
      )
    );
  }
}