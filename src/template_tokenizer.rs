#[derive(Debug, PartialEq)]
pub enum TemplateToken<'a> {
    Text(&'a str),
    Var(Path<'a>),
    For(&'a str, &'a str),
    EndFor(&'a str),
    If(Vec<&'a str>),
    EndIf,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Path<'a> {
  pub segments: Vec<&'a str>
}

pub fn tokenize<'a>(input: &'a str) -> Result<Vec<TemplateToken<'a>>, String> {
  let mut tokens = Vec::new();
  let mut rest = input;
  while !rest.is_empty() {
    if let Some(from) = rest.find('[') {
      if from > 0 {
        let text = &rest[..from];
        let text = TemplateToken::Text(text);
        println!("text: {:?}", text);
        tokens.push(text);
      }
      if let Some(to) = rest.find(']') {
        let tag_input = &rest[from+1..to];
        println!("tag input: '{}'", tag_input);
        let tag = parse_tag(tag_input)?;
        println!("tag: {:?}", tag);
        tokens.push(tag);
        rest = &rest[to+1..];
      }
      else {
        return Err("Missing closing bracket.".to_string());
      }
    }
    else {
      let text = rest;
      let text = TemplateToken::Text(text);
      println!("text: {:?}", text);
      tokens.push(text);
      break;
    }
  }
  Ok(tokens)
}

fn parse_tag<'a>(input: &'a str) -> Result<TemplateToken<'a>, String> {
  let parts: Vec<&str> = input.split_whitespace().collect();
  if parts.is_empty() {
    return Err("Tags cannot be empty.".to_string());
  }
  let tag = match parts[0] {
    "for" => parse_for_tag(parts)?,
    "endfor" => parse_endfor_tag(parts)?,
    "if" => parse_if_tag(parts)?,
    "endif" => parse_endif_tag(parts)?,
    _ => parse_var_tag(parts)?,
  };
  Ok(tag)
}

fn parse_var_tag(parts: Vec<&str>) -> Result<TemplateToken, &str> {
  if parts.len() == 1 {
    let segments = parts[0].split('.').collect::<Vec<&str>>();
    Ok(TemplateToken::Var(
      Path {
        segments: segments
      }
    ))
  }
  else {
    Err("Invalid var syntax - no parameters expected.")
  }
}

fn parse_endif_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
  assert!(parts[0] == "endif", "Expected 'endif' tag, got: {}", parts[0]);
  if parts.len() == 1 {
    Ok(TemplateToken::EndIf)
  }
  else {
    Err("Invalid endif tag syntax. No parameters expected.".to_string())
  }
}

fn parse_if_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
  assert!(parts[0] == "if", "Expected 'if' tag, got: {}", parts[0]);
  if parts.len() > 1 {
    Ok(TemplateToken::If(parts[1..].to_vec()))
  }
  else {
    Err("Invalid if tag syntax. Missing expression.".to_string())
  }
}

fn parse_for_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
  assert!(parts[0] == "for", "Expected 'for' tag, got: {}", parts[0]);
  if parts.len() == 4 {
    if parts[2] == "in" {
      Ok(TemplateToken::For(parts[1], parts[3]))
    }
    else {
      Err("Invalid for tag syntax. Missing 'in' keyword.".to_string())
    }
  }
  else {
    Err(
      format!(
        "Invalid for tag syntax. Incorrect number of parts - expected 4 (for, var, in, expression), got {:?}.",
        parts
      )
    )
  }
}

fn parse_endfor_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
  assert!(parts[0] == "endfor", "Expected 'endfor' tag, got: {}", parts[0]);
  if parts.len() == 2 {
    Ok(TemplateToken::EndFor(parts[1]))
  }
  else {
    Err(
      format!(
        "Invalid endfor tag syntax. Incorrect number of parts - expected 2 (endfor, var), got {:?}.",
        parts
      )
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::TemplateToken::*;

  #[test]
  fn tokenize_handles_text() {
    let result = tokenize("Hello, world!").unwrap();
    assert_eq!(
      vec![Text("Hello, world!")],
      result
    );
  }

  #[test]
  fn tokenize_handles_var_with_simple_path() {
    let result = tokenize("[title]").unwrap();
    assert_eq!(
      vec![
        Var(
          Path {
            segments: vec!["title"]
          }
        )
      ],
      result
    );
  }

  #[test]
  fn tokenize_handles_var_with_complex_path() {
    let result = tokenize("[section.title]").unwrap();
    assert_eq!(
      vec![
        Var(
          Path {
            segments: vec!["section", "title"]
          }
        )
      ],
      result
    );
  }

  #[test]
  fn tokenize_fails_if_no_closing_bracket() {
    let result = tokenize("[section.title").unwrap_err();
    assert!(result.to_string().contains("Missing closing bracket."));
  }

  #[test]
  fn tokenize_handles_mixed_text_and_var() {
    let result = tokenize("Hello, [section.title]!").unwrap();
    assert_eq!(
      vec![
        Text("Hello, "),
        Var(
          Path {
            segments: vec!["section", "title"]
          }),
        Text("!")
      ],
      result
    );
  }

  #[test]
  fn tokenize_handles_for_endfor() {
    let result = tokenize("\
[ for content in section.content ]
  Some text.
[ endfor content ]").unwrap();
    assert_eq!(
      vec![
        For("content", "section.content"),
        Text("\n  Some text.\n"),
        EndFor("content")
      ],
      result
    );
  }

  #[test]
  fn tokenize_fails_if_for_syntax_is_invalid() {
    let error = "Invalid for tag syntax.";
    assert_invalid_syntax("[ for ]", error);
    assert_invalid_syntax("[ for section in ]", error);
    assert_invalid_syntax("[ for in section.content ]", error);
    assert_invalid_syntax("[ for content section.content ]", error);
    assert_invalid_syntax("[ for content : section.content ]", error);
  }

  #[test]
  fn tokenize_fails_if_endfor_syntax_is_invalid() {
    let error = "Invalid endfor tag syntax.";
    assert_invalid_syntax("[ endfor ]", error);
    assert_invalid_syntax("[ endfor content extra ]", error);
  }

  #[test]
  fn tokenize_handles_if_endif() {
    let result = tokenize("\
[ if exists section.subsections ]
  Some text.
[ endif ]").unwrap();
    assert_eq!(
      vec![
        If(vec!["exists", "section.subsections"]),
        Text("\n  Some text.\n"),
        EndIf
      ],
      result
    );
  }

  #[test]
  fn tokenize_if_requires_an_expression() {
    assert_invalid_syntax("[ if ]", "Invalid if tag syntax.");
  }

  #[test]
  fn tokenize_endif_doesnt_support_expressions() {
    assert_invalid_syntax("[ endif expression ]", "Invalid endif tag syntax.");
  }

  #[test]
  fn tokenize_doesnt_support_empty_tags() {
    assert_invalid_syntax("[  ]", "Tags cannot be empty.");
  }

  fn assert_invalid_syntax(input: &str, expected: &str) {
    let err = super::tokenize(input).unwrap_err();
    assert!(err.contains(expected),
      "Expected error for input '{}', got: {}", input, err);
  }
}
