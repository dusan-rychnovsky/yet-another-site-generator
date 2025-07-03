use crate::template_tokenizer::{self, TemplateToken};
use crate::expressions::{Path, Expr};
use std::option::Option::{self, Some, None};

#[derive(Debug, PartialEq)]
pub struct TemplateTree<'a> {
  pub root: TemplateNode<'a>
}

#[derive(Debug, PartialEq)]
pub enum TemplateNode<'a> {
  Seq(Vec<Box<TemplateNode<'a>>>),
  Text(&'a str),
  Var(Path<'a>),
  ForEach (&'a str, Path<'a>, Box<TemplateNode<'a>>),
  If (Expr<'a>, Box<TemplateNode<'a>>)
}

pub fn parse<'a>(input: &'a str) -> Result<TemplateTree<'a>, String> {
  let tokens = template_tokenizer::tokenize(input)
    .map_err(|e| format!("Failed to tokenize template: {}", e))?;
  let (nodes, _) = parse_nodes(&tokens, 0, None)?;
  Ok(TemplateTree {
    root: TemplateNode::Seq(nodes),
  })
}

fn parse_nodes<'a>(tokens: &[TemplateToken<'a>], start_pos: usize, context: Option<&TemplateToken<'a>>)
  -> Result<(Vec<Box<TemplateNode<'a>>>, usize), String> {

  let mut nodes = Vec::new();
  let mut pos = start_pos;
  while pos < tokens.len() {
    let token = &tokens[pos];
    pos += 1;
    match token {
      TemplateToken::Text(text) => {
        nodes.push(Box::new(TemplateNode::Text(text)));
      }
      TemplateToken::Var(var) => {
        nodes.push(Box::new(TemplateNode::Var(var.clone())));
      }
      TemplateToken::For(var, expr) => {
        let (body, new_start_pos) = parse_nodes(tokens, pos, Some(token))?;
        nodes.push(Box::new(TemplateNode::ForEach(
          var,
          expr.clone(),
          Box::new(TemplateNode::Seq(body))
        )));
        pos = new_start_pos;
      }
      TemplateToken::EndFor(var) => {
        match context {
          Some(TemplateToken::For(ctx_var, _)) => {
            if ctx_var == var {
              break;
            }
          },
          _ => ()
        };
        return Err(format!("Unexpected token EndFor(\"{}\") nested in {:?}.", var, context));
      },
      TemplateToken::If(cond) => {
        let (body, new_start_pos) = parse_nodes(tokens, pos, Some(token))?;
        nodes.push(Box::new(TemplateNode::If(
          cond.clone(),
          Box::new(TemplateNode::Seq(body))
        )));
        pos = new_start_pos;
      }
      TemplateToken::EndIf => {
        match context {
          Some(TemplateToken::If(_)) => {
            break;
          },
          _ => {
            return Err(format!("Unexpected token EndIf nested in {:?}.", context));
          }
        }
      }
    }
  }
  Ok((nodes, pos))
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::TemplateNode::*;
  use crate::expressions::{Path, Expr, Predicate::*};

  #[test]
  fn parse_handles_empty_input() {
    let result = parse("").unwrap();
    assert_eq!(result.root, Seq(Vec::new()));
  }

  #[test]
  fn parse_handles_simple_text() {
    let input = "This is a simple text.";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(Text(input))
        ]
      )
    );
  }

  #[test]
  fn parse_handles_text_with_variables() {
    let input = "Hello, [name]! Welcome to [place.address].";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(Text("Hello, ")),
          Box::new(Var(Path::from(vec!["name"]))),
          Box::new(Text("! Welcome to ")),
          Box::new(Var(Path::from(vec!["place", "address"]))),
          Box::new(Text("."))
        ]
      )
    );
  }

  #[test]
  fn parse_handles_foreach() {
    let input = "\
[for section in sections]
  Section. Title: [section.title]
[endfor section]";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(ForEach(
            "section",
            Path { segments: vec!["sections"] },
            Box::new(Seq(vec![
              Box::new(Text("\n  Section. Title: ")),
              Box::new(Var(Path::from(vec!["section", "title"]))),
              Box::new(Text("\n"))
            ]))
          ))
        ]
      )
    );
  }

  #[test]
  fn parse_handles_nested_foreach() {
    let input = "\
[for section in sections]
  <ul>
    [for link in section.links]
      <li>
        Link: [link.href]
      </li>
    [endfor link]
  </ul>
[endfor section]";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(ForEach(
            "section",
            Path { segments: vec!["sections"] },
            Box::new(Seq(vec![
              Box::new(Text("\n  <ul>\n    ")),
              Box::new(ForEach(
                "link",
                Path::from(vec!["section", "links"]),
                Box::new(Seq(vec![
                  Box::new(Text("\n      <li>\n        Link: ")),
                  Box::new(Var(Path::from(vec!["link", "href"]))),
                  Box::new(Text("\n      </li>\n    "))
                ]))
              )),
              Box::new(Text("\n  </ul>\n"))
            ]))
          ))
        ]
      )
    );
  }

  #[test]
  fn parse_nested_foreach_with_incorrect_closing_order_fails() {
    let input = "[for section in sections]
      <ul>
        [for link in section.links]
          <li>
            Link: [link.href]
          </li>
        [endfor section]
      </ul>
    [endfor link]";
    assert_invalid_syntax(
      input,
      "Unexpected token EndFor(\"section\") nested in Some(For(\"link\", Path { segments: [\"section\", \"links\"] }))."
    );
  }

  #[test]
  fn parse_endfor_without_for_fails() {
    let input = "[endfor section]";
    assert_invalid_syntax(input, "Unexpected token EndFor(\"section\") nested in None.");
  }

  #[test]
  fn parse_handles_if_statements() {
    let input = "\
[if exists section.subsections]
  Subsections exist.
[endif]";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(If(
            Expr::from(Exists, vec!["section", "subsections"]),
            Box::new(Seq(vec![
              Box::new(Text("\n  Subsections exist.\n"))
            ]))
          ))
        ]
      )
    );
  }

  #[test]
  fn parse_handles_foreach_nested_in_if() {
    let input = "\
[if exists section.subsections]
  <ul>
    [for subsection in section.subsections]
      <li>Subsection: [subsection.title]</li>
    [endfor subsection]
  </ul>
[endif]";
    let result = parse(input).unwrap();
    assert_eq!(
      result.root,
      Seq(
        vec![
          Box::new(If(
            Expr::from(Exists, vec!["section", "subsections"]),
            Box::new(Seq(vec![
              Box::new(Text("\n  <ul>\n    ")),
              Box::new(ForEach(
                "subsection",
                Path::from(vec!["section", "subsections"]),
                Box::new(Seq(vec![
                  Box::new(Text("\n      <li>Subsection: ")),
                  Box::new(Var(Path::from(vec!["subsection", "title"]))),
                  Box::new(Text("</li>\n    "))
                ]))
              )),
              Box::new(Text("\n  </ul>\n"))
            ]))
          ))
        ]
      )
    );
  } 

  #[test]
  fn parse_with_incorrect_if_and_foreach_nesting_fails() {
    let input = "\
[if exists section.subsections]
  <ul>
    [for subsection in section.subsections]
      <li>Subsection: [subsection.title]</li>
    [endif]
  </ul>
[endfor subsection]";
    assert_invalid_syntax(
      input,
      "Unexpected token EndIf nested in Some(For(\"subsection\", Path { segments: [\"section\", \"subsections\"] }))."
    );
  }

  #[test]
  fn parse_endif_without_if_fails() {
    let input = "[endif]";
    assert_invalid_syntax(input, "Unexpected token EndIf nested in None.");
  }

  fn assert_invalid_syntax(input: &str, expected: &str) {
    let err = parse(input).unwrap_err();
    assert!(err.contains(expected),
      "Expected error for input '{}', got: {}", input, err);
  }
}