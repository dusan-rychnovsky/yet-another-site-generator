use crate::expressions::{Expr, Path};

/// Represents a lexical token of a template file.
#[derive(Debug, PartialEq, Clone)]
pub enum TemplateToken {
    Text(String),
    Var(Path),
    Func(String, Vec<Path>),
    For(String, Path),
    EndFor(String),
    If(Expr),
    EndIf,
    Include(String),
}

/// Tokenizes the given template file into a sequence of [`TemplateToken`].
pub fn tokenize(input: &str) -> Result<Vec<TemplateToken>, String> {
    let mut tokens = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        if let Some(from) = rest.find('[') {
            if from > 0 {
                let text = &rest[..from];
                let text = TemplateToken::Text(text.to_string());
                tokens.push(text);
            }
            if let Some(to) = rest.find(']') {
                let tag_input = &rest[from + 1..to];
                let tag = TemplateToken::parse_tag(tag_input)?;
                tokens.push(tag);
                rest = &rest[to + 1..];
            } else {
                return Err("Missing closing bracket.".to_string());
            }
        } else {
            let text = rest;
            let text = TemplateToken::Text(text.to_string());
            tokens.push(text);
            break;
        }
    }
    Ok(tokens)
}

impl TemplateToken {
    /// Parses the given string into a [`TemplateToken`].
    fn parse_tag(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Tags cannot be empty.".to_string());
        }
        let tag = match parts[0] {
            "INCLUDE" => Self::parse_include_tag(parts)?,
            "FOR" => Self::parse_for_tag(parts)?,
            "ENDFOR" => Self::parse_endfor_tag(parts)?,
            "IF" => Self::parse_if_tag(parts)?,
            "ENDIF" => Self::parse_endif_tag(parts)?,
            _ if input.contains('(') => Self::parse_func_tag(input.trim())?,
            _ => Self::parse_var_tag(parts)?,
        };
        Ok(tag)
    }

    /// Parses the given string into a [`TemplateToken::Func`]. Expected syntax:
    /// `name(arg1, arg2, ...)`, where each argument is a [`Path`]. Whitespace around the name and
    /// arguments is ignored.
    fn parse_func_tag(input: &str) -> Result<TemplateToken, String> {
        let open = input
            .find('(')
            .ok_or_else(|| format!("Invalid function syntax. Missing '(': '{}'.", input))?;
        if !input.ends_with(')') {
            return Err(format!(
                "Invalid function syntax. Missing closing ')': '{}'.",
                input
            ));
        }
        let name = input[..open].trim();
        if name.is_empty() {
            return Err(format!(
                "Invalid function syntax. Missing function name: '{}'.",
                input
            ));
        }
        let args_str = input[open + 1..input.len() - 1].trim();
        let args = if args_str.is_empty() {
            Vec::new()
        } else {
            args_str
                .split(',')
                .map(|arg| Path::parse(arg.trim()))
                .collect()
        };
        Ok(TemplateToken::Func(name.to_string(), args))
    }

    /// Parses the given sequence of strings into a [`TemplateToken::Var`].
    fn parse_var_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        if parts.len() == 1 {
            Ok(TemplateToken::Var(Path::parse(parts[0])))
        } else {
            Err(format!(
                "Invalid var syntax. No parameters expected. Parts: {:?}",
                parts
            ))
        }
    }

    /// Parses the given sequence of strings into a [`TemplateToken::EndIf`].
    fn parse_endif_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        assert!(
            parts[0] == "ENDIF",
            "Expected 'ENDIF' tag, got: {}",
            parts[0]
        );
        if parts.len() == 1 {
            Ok(TemplateToken::EndIf)
        } else {
            Err(format!(
                "Invalid endif tag syntax. No parameters expected. Parts: {:?}",
                parts
            ))
        }
    }

    /// Parses the given sequence of strings into a [`TemplateToken::If`].
    fn parse_if_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        assert!(parts[0] == "IF", "Expected 'IF' tag, got: {}", parts[0]);
        let expr = Expr::parse(parts[1..].to_vec())
            .map_err(|e| format!("Invalid if tag syntax: {}", e))?;
        Ok(TemplateToken::If(expr))
    }

    /// Parses the given sequence of strings into a [`TemplateToken::For`].
    fn parse_for_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        assert!(parts[0] == "FOR", "Expected 'FOR' tag, got: {}", parts[0]);
        if parts.len() == 4 {
            if parts[2] == "IN" {
                Ok(TemplateToken::For(
                    parts[1].to_string(),
                    Path::parse(parts[3]),
                ))
            } else {
                Err("Invalid for tag syntax. Missing 'IN' keyword.".to_string())
            }
        } else {
            Err(format!(
                "Invalid for tag syntax. Incorrect number of parts - expected 4 (FOR, var, IN, expression), got {:?}.",
                parts
            ))
        }
    }

    /// Parses the given sequence of strings into a [`TemplateToken::EndFor`].
    fn parse_endfor_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        assert!(
            parts[0] == "ENDFOR",
            "Expected 'ENDFOR' tag, got: {}",
            parts[0]
        );
        if parts.len() == 2 {
            Ok(TemplateToken::EndFor(parts[1].to_string()))
        } else {
            Err(format!(
                "Invalid endfor tag syntax. Incorrect number of parts - expected 2 (ENDFOR, var), got {:?}.",
                parts
            ))
        }
    }

    /// Parses the given sequence of strings into a [`TemplateToken::Include`].
    fn parse_include_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
        assert!(
            parts[0] == "INCLUDE",
            "Expected 'INCLUDE' tag, got: {}",
            parts[0]
        );
        if parts.len() == 2 {
            Ok(TemplateToken::Include(parts[1].to_string()))
        } else {
            Err(format!(
                "Invalid include tag syntax. Incorrect number of parts - expected 2 (INCLUDE, path), got {:?}.",
                parts
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TemplateToken::*;
    use super::*;
    use crate::expressions::{Expr, Path, Predicate::*};

    #[test]
    fn tokenize_handles_text() {
        let result = tokenize("Hello, world!").unwrap();
        assert_eq!(vec![Text("Hello, world!".to_string())], result);
    }

    #[test]
    fn tokenize_handles_var_with_simple_path() {
        let result = tokenize("[title]").unwrap();
        assert_eq!(vec![Var(Path::from_segment("title"))], result);
    }

    #[test]
    fn tokenize_handles_var_with_complex_path() {
        let result = tokenize("[section.title]").unwrap();
        assert_eq!(
            vec![Var(Path::from_segments(vec!["section", "title"]))],
            result
        );
    }

    #[test]
    fn tokenize_handles_func_with_arguments() {
        let result = tokenize("[LINK(PATH, page.PATH)]").unwrap();
        assert_eq!(
            vec![Func(
                "LINK".to_string(),
                vec![
                    Path::from_segment("PATH"),
                    Path::from_segments(vec!["page", "PATH"]),
                ]
            )],
            result
        );
    }

    #[test]
    fn tokenize_handles_func_with_no_arguments() {
        let result = tokenize("[now()]").unwrap();
        assert_eq!(vec![Func("now".to_string(), Vec::new())], result);
    }

    #[test]
    fn tokenize_fails_if_func_has_no_closing_paren() {
        assert_invalid_syntax("[LINK(PATH, page.PATH]", "Missing closing ')'");
    }

    #[test]
    fn tokenize_fails_if_func_has_no_name() {
        assert_invalid_syntax("[(PATH, page.PATH)]", "Missing function name");
    }

    #[test]
    fn tokenize_fails_if_no_closing_bracket() {
        let result = tokenize("[section.title").unwrap_err();
        assert!(result.contains("Missing closing bracket."));
    }

    #[test]
    fn tokenize_handles_mixed_text_and_var() {
        let result = tokenize("Hello, [section.title]!").unwrap();
        assert_eq!(
            vec![
                Text("Hello, ".to_string()),
                Var(Path::from_segments(vec!["section", "title"])),
                Text("!".to_string())
            ],
            result
        );
    }

    #[test]
    fn tokenize_handles_for_endfor() {
        let result = tokenize(
            "\
[ FOR content IN section.content ]
  Some text.
[ ENDFOR content ]",
        )
        .unwrap();
        assert_eq!(
            vec![
                For(
                    "content".to_string(),
                    Path::from_segments(vec!["section", "content"])
                ),
                Text("\n  Some text.\n".to_string()),
                EndFor("content".to_string())
            ],
            result
        );
    }

    #[test]
    fn tokenize_fails_if_for_syntax_is_invalid() {
        let error = "Invalid for tag syntax.";
        assert_invalid_syntax("[ FOR ]", error);
        assert_invalid_syntax("[ FOR section IN ]", error);
        assert_invalid_syntax("[ FOR IN section.content ]", error);
        assert_invalid_syntax("[ FOR content section.content ]", error);
        assert_invalid_syntax("[ FOR content : section.content ]", error);
    }

    #[test]
    fn tokenize_fails_if_endfor_syntax_is_invalid() {
        let error = "Invalid endfor tag syntax.";
        assert_invalid_syntax("[ ENDFOR ]", error);
        assert_invalid_syntax("[ ENDFOR content extra ]", error);
    }

    #[test]
    fn tokenize_handles_if_endif() {
        let result = tokenize(
            "\
[ IF EXISTS section.subsections ]
  Some text.
[ ENDIF ]",
        )
        .unwrap();
        assert_eq!(
            vec![
                If(Expr::from(Exists, vec!["section", "subsections"])),
                Text("\n  Some text.\n".to_string()),
                EndIf
            ],
            result
        );
    }

    #[test]
    fn tokenize_if_requires_an_expression() {
        assert_invalid_syntax(
            "[ IF ]",
            "Invalid if tag syntax: Invalid expression syntax - expected a predicate and a path, got: '[]'.",
        );
    }

    #[test]
    fn tokenize_endif_doesnt_support_expressions() {
        assert_invalid_syntax("[ ENDIF expression ]", "Invalid endif tag syntax.");
    }

    #[test]
    fn tokenize_doesnt_support_empty_tags() {
        assert_invalid_syntax("[  ]", "Tags cannot be empty.");
    }

    #[test]
    fn tokenize_handles_include() {
        let result = tokenize("[ INCLUDE snippets/menu.html ]").unwrap();
        assert_eq!(vec![Include("snippets/menu.html".to_string())], result);
    }

    fn assert_invalid_syntax(input: &str, expected: &str) {
        let err = super::tokenize(input).unwrap_err();
        assert!(
            err.contains(expected),
            "Expected error for input '{}', got: {}",
            input,
            err
        );
    }
}
