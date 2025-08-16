use crate::expressions::{Expr, Path};
use crate::template_tokenizer::TemplateToken;
use std::option::Option::{self, None, Some};

/// Represents a parse tree of a template file.
#[derive(Debug, PartialEq)]
pub struct TemplateTree<'a> {
    pub root: TemplateNode<'a>,
}

/// Represents a node in a [`TemplateTree`].
#[derive(Debug, PartialEq)]
pub enum TemplateNode<'a> {
    Seq(Vec<Box<TemplateNode<'a>>>),
    Text(&'a str),
    Var(Path<'a>),
    ForEach(&'a str, Path<'a>, Box<TemplateNode<'a>>),
    If(Expr<'a>, Box<TemplateNode<'a>>),
}

/// Parses the given sequence of tokens into a parse tree.
pub fn parse<'a>(tokens: &[TemplateToken<'a>]) -> Result<TemplateTree<'a>, String> {
    let (nodes, _) = parse_nodes(tokens, 0, None)?;
    Ok(TemplateTree {
        root: TemplateNode::Seq(nodes),
    })
}

/// Parses the given sequence of tokens into a parse tree, starting from the given position.
/// This is a helper function for handling recursion.
fn parse_nodes<'a>(
    tokens: &[TemplateToken<'a>],
    start_pos: usize,
    context: Option<&TemplateToken<'a>>,
) -> Result<(Vec<Box<TemplateNode<'a>>>, usize), String> {
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
                    Box::new(TemplateNode::Seq(body)),
                )));
                pos = new_start_pos;
            }
            TemplateToken::EndFor(var) => {
                if let Some(TemplateToken::For(ctx_var, _)) = context {
                    if ctx_var == var {
                        break;
                    }
                }
                return Err(format!(
                    "Unexpected token EndFor(\"{}\") nested in {:?}.",
                    var, context
                ));
            }
            TemplateToken::If(cond) => {
                let (body, new_start_pos) = parse_nodes(tokens, pos, Some(token))?;
                nodes.push(Box::new(TemplateNode::If(
                    cond.clone(),
                    Box::new(TemplateNode::Seq(body)),
                )));
                pos = new_start_pos;
            }
            TemplateToken::EndIf => {
                if let Some(TemplateToken::If(_)) = context {
                    break;
                } else {
                    return Err(format!("Unexpected token EndIf nested in {:?}.", context));
                }
            }
        }
    }
    Ok((nodes, pos))
}

#[cfg(test)]
mod tests {
    use super::TemplateNode::*;
    use super::*;
    use crate::expressions::{Expr, Path, Predicate::*};
    use std::vec;

    #[test]
    fn parse_handles_empty_input() {
        let tokens = vec![]; // TODO: is it empty vector or a Text with empty string?
        let result = parse(&tokens).unwrap();
        assert_eq!(result.root, Seq(Vec::new()));
    }

    #[test]
    fn parse_handles_simple_text() {
        let text = "This is a simple text.";
        let tokens = vec![TemplateToken::Text(text)];
        let result = parse(&tokens).unwrap();
        assert_eq!(result.root, Seq(vec![Box::new(Text(text))]));
    }

    #[test]
    fn parse_handles_text_with_variables() {
        let tokens = vec![
            TemplateToken::Text("Hello, "),
            TemplateToken::Var(Path::from_segment("name")),
            TemplateToken::Text("! Welcome to "),
            TemplateToken::Var(Path::from_segments(vec!["place", "address"])),
            TemplateToken::Text("."),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![
                Box::new(Text("Hello, ")),
                Box::new(Var(Path::from_segment("name"))),
                Box::new(Text("! Welcome to ")),
                Box::new(Var(Path::from_segments(vec!["place", "address"]))),
                Box::new(Text("."))
            ])
        );
    }

    #[test]
    fn parse_handles_foreach() {
        let tokens = vec![
            TemplateToken::For("section", Path::from_segment("sections")),
            TemplateToken::Text("\n  Section. Title: "),
            TemplateToken::Var(Path::from_segments(vec!["section", "title"])),
            TemplateToken::Text("\n"),
            TemplateToken::EndFor("section"),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![Box::new(ForEach(
                "section",
                Path {
                    segments: vec!["sections"]
                },
                Box::new(Seq(vec![
                    Box::new(Text("\n  Section. Title: ")),
                    Box::new(Var(Path::from_segments(vec!["section", "title"]))),
                    Box::new(Text("\n"))
                ]))
            ))])
        );
    }

    #[test]
    fn parse_handles_nested_foreach() {
        let tokens = vec![
            TemplateToken::For("section", Path::from_segment("sections")),
            TemplateToken::Text("\n  <ul>\n    "),
            TemplateToken::For("link", Path::from_segments(vec!["section", "links"])),
            TemplateToken::Text("\n      <li>\n        Link: "),
            TemplateToken::Var(Path::from_segments(vec!["link", "href"])),
            TemplateToken::Text("\n      </li>\n    "),
            TemplateToken::EndFor("link"),
            TemplateToken::Text("\n  </ul>\n"),
            TemplateToken::EndFor("section"),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![Box::new(ForEach(
                "section",
                Path {
                    segments: vec!["sections"]
                },
                Box::new(Seq(vec![
                    Box::new(Text("\n  <ul>\n    ")),
                    Box::new(ForEach(
                        "link",
                        Path::from_segments(vec!["section", "links"]),
                        Box::new(Seq(vec![
                            Box::new(Text("\n      <li>\n        Link: ")),
                            Box::new(Var(Path::from_segments(vec!["link", "href"]))),
                            Box::new(Text("\n      </li>\n    "))
                        ]))
                    )),
                    Box::new(Text("\n  </ul>\n"))
                ]))
            ))])
        );
    }

    #[test]
    fn parse_nested_foreach_with_incorrect_closing_order_fails() {
        assert_invalid_syntax(
            &vec![
                TemplateToken::For("section", Path::from_segment("sections")),
                TemplateToken::Text("\n      <ul>\n        "),
                TemplateToken::For("link", Path::from_segments(vec!["section", "links"])),
                TemplateToken::Text("\n          <li>\n            Link: "),
                TemplateToken::Var(Path::from_segments(vec!["link", "href"])),
                TemplateToken::Text("\n          </li>\n        "),
                TemplateToken::EndFor("section"),
                TemplateToken::Text("\n      </ul>\n    "),
                TemplateToken::EndFor("link"),
            ],
            "Unexpected token EndFor(\"section\") nested in Some(For(\"link\", Path { segments: [\"section\", \"links\"] })).",
        );
    }

    #[test]
    fn parse_endfor_without_for_fails() {
        assert_invalid_syntax(
            &vec![TemplateToken::EndFor("section")],
            "Unexpected token EndFor(\"section\") nested in None.",
        );
    }

    #[test]
    fn parse_handles_if_statements() {
        let tokens = vec![
            TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
            TemplateToken::Text("\n  Subsections exist.\n"),
            TemplateToken::EndIf,
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![Box::new(If(
                Expr::from(Exists, vec!["section", "subsections"]),
                Box::new(Seq(vec![Box::new(Text("\n  Subsections exist.\n"))]))
            ))])
        );
    }

    #[test]
    fn parse_handles_foreach_nested_in_if() {
        let tokens = vec![
            TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
            TemplateToken::Text("\n  <ul>\n    "),
            TemplateToken::For(
                "subsection",
                Path::from_segments(vec!["section", "subsections"]),
            ),
            TemplateToken::Text("\n      <li>Subsection: "),
            TemplateToken::Var(Path::from_segments(vec!["subsection", "title"])),
            TemplateToken::Text("</li>\n    "),
            TemplateToken::EndFor("subsection"),
            TemplateToken::Text("\n  </ul>\n"),
            TemplateToken::EndIf,
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![Box::new(If(
                Expr::from(Exists, vec!["section", "subsections"]),
                Box::new(Seq(vec![
                    Box::new(Text("\n  <ul>\n    ")),
                    Box::new(ForEach(
                        "subsection",
                        Path::from_segments(vec!["section", "subsections"]),
                        Box::new(Seq(vec![
                            Box::new(Text("\n      <li>Subsection: ")),
                            Box::new(Var(Path::from_segments(vec!["subsection", "title"]))),
                            Box::new(Text("</li>\n    "))
                        ]))
                    )),
                    Box::new(Text("\n  </ul>\n"))
                ]))
            ))])
        );
    }

    #[test]
    fn parse_with_incorrect_if_and_foreach_nesting_fails() {
        assert_invalid_syntax(
            &vec![
                TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
                TemplateToken::Text("\n  <ul>\n    "),
                TemplateToken::For(
                    "subsection",
                    Path::from_segments(vec!["section", "subsections"]),
                ),
                TemplateToken::Text("\n      <li>Subsection: "),
                TemplateToken::Var(Path::from_segments(vec!["subsection", "title"])),
                TemplateToken::Text("</li>\n    "),
                TemplateToken::EndIf, // This should be EndFor
                TemplateToken::Text("\n  </ul>\n"),
                TemplateToken::EndFor("subsection"),
            ],
            "Unexpected token EndIf nested in Some(For(\"subsection\", Path { segments: [\"section\", \"subsections\"] })).",
        );
    }

    #[test]
    fn parse_endif_without_if_fails() {
        assert_invalid_syntax(
            &vec![TemplateToken::EndIf],
            "Unexpected token EndIf nested in None.",
        );
    }

    fn assert_invalid_syntax(tokens: &Vec<TemplateToken>, expected: &str) {
        let err = parse(tokens).unwrap_err();
        assert!(
            err.contains(expected),
            "Expected error for input '{:#?}', got: {}",
            tokens,
            err
        );
    }
}
