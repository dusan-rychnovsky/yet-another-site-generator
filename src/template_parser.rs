use crate::expressions::{Expr, Path};
use crate::template_tokenizer::TemplateToken;
use std::option::Option::{self, None, Some};

/// Represents a parse tree of a template file.
#[derive(Debug, PartialEq)]
pub struct TemplateTree {
    pub root: TemplateNode,
}

/// Represents a node in a [`TemplateTree`].
#[derive(Debug, PartialEq)]
pub enum TemplateNode {
    Seq(Vec<TemplateNode>),
    Text(String),
    Var(Path),
    Func(String, Vec<Path>),
    ForEach(String, Path, Box<TemplateNode>),
    If(Expr, Box<TemplateNode>),
}

/// Parses the given sequence of tokens into a parse tree.
pub fn parse(tokens: &[TemplateToken]) -> Result<TemplateTree, String> {
    let (nodes, _) = parse_nodes(tokens, 0, None)?;
    Ok(TemplateTree {
        root: TemplateNode::Seq(nodes),
    })
}

/// Parses the given sequence of tokens into a parse tree, starting from the given position.
/// This is a helper function for handling recursion.
fn parse_nodes(
    tokens: &[TemplateToken],
    start_pos: usize,
    context: Option<&TemplateToken>,
) -> Result<(Vec<TemplateNode>, usize), String> {
    let mut nodes = Vec::new();
    let mut pos = start_pos;
    while pos < tokens.len() {
        let token = &tokens[pos];
        pos += 1;
        match token {
            TemplateToken::Text(text) => {
                nodes.push(TemplateNode::Text(text.clone()));
            }
            TemplateToken::Var(var) => {
                nodes.push(TemplateNode::Var(var.clone()));
            }
            TemplateToken::Func(name, args) => {
                nodes.push(TemplateNode::Func(name.clone(), args.clone()));
            }
            TemplateToken::For(var, expr) => {
                let (body, new_start_pos) = parse_nodes(tokens, pos, Some(token))?;
                nodes.push(TemplateNode::ForEach(
                    var.clone(),
                    expr.clone(),
                    Box::new(TemplateNode::Seq(body)),
                ));
                pos = new_start_pos;
            }
            TemplateToken::EndFor(var) => match context {
                Some(TemplateToken::For(ctx_var, _)) if ctx_var == var => break,
                _ => {
                    return Err(format!(
                        "Unexpected token EndFor(\"{}\") nested in {:?}.",
                        var, context
                    ));
                }
            },
            TemplateToken::If(cond) => {
                let (body, new_start_pos) = parse_nodes(tokens, pos, Some(token))?;
                nodes.push(TemplateNode::If(
                    cond.clone(),
                    Box::new(TemplateNode::Seq(body)),
                ));
                pos = new_start_pos;
            }
            TemplateToken::EndIf => {
                if let Some(TemplateToken::If(_)) = context {
                    break;
                } else {
                    return Err(format!("Unexpected token EndIf nested in {:?}.", context));
                }
            }
            TemplateToken::Include(_) => {
                panic!("Include tokens need to be handled during tokenization.");
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
        let tokens = vec![TemplateToken::Text(text.to_string())];
        let result = parse(&tokens).unwrap();
        assert_eq!(result.root, Seq(vec![Text(text.to_string())]));
    }

    #[test]
    fn parse_handles_text_with_variables() {
        let tokens = vec![
            TemplateToken::Text("Hello, ".to_string()),
            TemplateToken::Var(Path::from_segment("name")),
            TemplateToken::Text("! Welcome to ".to_string()),
            TemplateToken::Var(Path::from_segments(vec!["place", "address"])),
            TemplateToken::Text(".".to_string()),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![
                Text("Hello, ".to_string()),
                Var(Path::from_segment("name")),
                Text("! Welcome to ".to_string()),
                Var(Path::from_segments(vec!["place", "address"])),
                Text(".".to_string())
            ])
        );
    }

    #[test]
    fn parse_handles_func() {
        let tokens = vec![
            TemplateToken::Text("<a href=\"".to_string()),
            TemplateToken::Func(
                "LINK".to_string(),
                vec![
                    Path::from_segment("PATH"),
                    Path::from_segments(vec!["page", "PATH"]),
                ],
            ),
            TemplateToken::Text("\">".to_string()),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![
                Text("<a href=\"".to_string()),
                Func(
                    "LINK".to_string(),
                    vec![
                        Path::from_segment("PATH"),
                        Path::from_segments(vec!["page", "PATH"]),
                    ]
                ),
                Text("\">".to_string()),
            ])
        );
    }

    #[test]
    fn parse_handles_foreach() {
        let tokens = vec![
            TemplateToken::For("section".to_string(), Path::from_segment("sections")),
            TemplateToken::Text("\n  Section. Title: ".to_string()),
            TemplateToken::Var(Path::from_segments(vec!["section", "title"])),
            TemplateToken::Text("\n".to_string()),
            TemplateToken::EndFor("section".to_string()),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![ForEach(
                "section".to_string(),
                Path::from_segment("sections"),
                Box::new(Seq(vec![
                    Text("\n  Section. Title: ".to_string()),
                    Var(Path::from_segments(vec!["section", "title"])),
                    Text("\n".to_string())
                ]))
            )])
        );
    }

    #[test]
    fn parse_handles_nested_foreach() {
        let tokens = vec![
            TemplateToken::For("section".to_string(), Path::from_segment("sections")),
            TemplateToken::Text("\n  <ul>\n    ".to_string()),
            TemplateToken::For(
                "link".to_string(),
                Path::from_segments(vec!["section", "links"]),
            ),
            TemplateToken::Text("\n      <li>\n        Link: ".to_string()),
            TemplateToken::Var(Path::from_segments(vec!["link", "href"])),
            TemplateToken::Text("\n      </li>\n    ".to_string()),
            TemplateToken::EndFor("link".to_string()),
            TemplateToken::Text("\n  </ul>\n".to_string()),
            TemplateToken::EndFor("section".to_string()),
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![ForEach(
                "section".to_string(),
                Path::from_segment("sections"),
                Box::new(Seq(vec![
                    Text("\n  <ul>\n    ".to_string()),
                    ForEach(
                        "link".to_string(),
                        Path::from_segments(vec!["section", "links"]),
                        Box::new(Seq(vec![
                            Text("\n      <li>\n        Link: ".to_string()),
                            Var(Path::from_segments(vec!["link", "href"])),
                            Text("\n      </li>\n    ".to_string())
                        ]))
                    ),
                    Text("\n  </ul>\n".to_string())
                ]))
            )])
        );
    }

    #[test]
    fn parse_nested_foreach_with_incorrect_closing_order_fails() {
        assert_invalid_syntax(
            &vec![
                TemplateToken::For("section".to_string(), Path::from_segment("sections")),
                TemplateToken::Text("\n      <ul>\n        ".to_string()),
                TemplateToken::For(
                    "link".to_string(),
                    Path::from_segments(vec!["section", "links"]),
                ),
                TemplateToken::Text("\n          <li>\n            Link: ".to_string()),
                TemplateToken::Var(Path::from_segments(vec!["link", "href"])),
                TemplateToken::Text("\n          </li>\n        ".to_string()),
                TemplateToken::EndFor("section".to_string()),
                TemplateToken::Text("\n      </ul>\n    ".to_string()),
                TemplateToken::EndFor("link".to_string()),
            ],
            "Unexpected token EndFor(\"section\") nested in Some(For(\"link\", Path { segments: [\"section\", \"links\"] })).",
        );
    }

    #[test]
    fn parse_endfor_without_for_fails() {
        assert_invalid_syntax(
            &vec![TemplateToken::EndFor("section".to_string())],
            "Unexpected token EndFor(\"section\") nested in None.",
        );
    }

    #[test]
    fn parse_handles_if_statements() {
        let tokens = vec![
            TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
            TemplateToken::Text("\n  Subsections exist.\n".to_string()),
            TemplateToken::EndIf,
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![If(
                Expr::from(Exists, vec!["section", "subsections"]),
                Box::new(Seq(vec![Text("\n  Subsections exist.\n".to_string())]))
            )])
        );
    }

    #[test]
    fn parse_handles_foreach_nested_in_if() {
        let tokens = vec![
            TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
            TemplateToken::Text("\n  <ul>\n    ".to_string()),
            TemplateToken::For(
                "subsection".to_string(),
                Path::from_segments(vec!["section", "subsections"]),
            ),
            TemplateToken::Text("\n      <li>Subsection: ".to_string()),
            TemplateToken::Var(Path::from_segments(vec!["subsection", "title"])),
            TemplateToken::Text("</li>\n    ".to_string()),
            TemplateToken::EndFor("subsection".to_string()),
            TemplateToken::Text("\n  </ul>\n".to_string()),
            TemplateToken::EndIf,
        ];
        let result = parse(&tokens).unwrap();
        assert_eq!(
            result.root,
            Seq(vec![If(
                Expr::from(Exists, vec!["section", "subsections"]),
                Box::new(Seq(vec![
                    Text("\n  <ul>\n    ".to_string()),
                    ForEach(
                        "subsection".to_string(),
                        Path::from_segments(vec!["section", "subsections"]),
                        Box::new(Seq(vec![
                            Text("\n      <li>Subsection: ".to_string()),
                            Var(Path::from_segments(vec!["subsection", "title"])),
                            Text("</li>\n    ".to_string())
                        ]))
                    ),
                    Text("\n  </ul>\n".to_string())
                ]))
            )])
        );
    }

    #[test]
    fn parse_with_incorrect_if_and_foreach_nesting_fails() {
        assert_invalid_syntax(
            &vec![
                TemplateToken::If(Expr::from(Exists, vec!["section", "subsections"])),
                TemplateToken::Text("\n  <ul>\n    ".to_string()),
                TemplateToken::For(
                    "subsection".to_string(),
                    Path::from_segments(vec!["section", "subsections"]),
                ),
                TemplateToken::Text("\n      <li>Subsection: ".to_string()),
                TemplateToken::Var(Path::from_segments(vec!["subsection", "title"])),
                TemplateToken::Text("</li>\n    ".to_string()),
                TemplateToken::EndIf, // This should be EndFor
                TemplateToken::Text("\n  </ul>\n".to_string()),
                TemplateToken::EndFor("subsection".to_string()),
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
