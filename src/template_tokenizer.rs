#[derive(Debug, PartialEq)]
pub enum TemplateToken {
  Text(String), // TODO: replace with string slice with lifetime
  Var(String),
  For(String, String),
  EndFor(String),
  If(String),
  EndIf
}

pub fn tokenize(path: &str) -> Result<Vec<TemplateToken>, Box<dyn std::error::Error>> {
  let content = std::fs::read_to_string(path)?;
  let tokens = tokenize_content(&content)?;
  Ok(tokens)
}

/*
let mut tokens = Vec::new();
  let mut rest = input;
  while !rest.is_empty() {
    if let Some(start) = rest.find("[") {
      if start > 0 {
        tokens.push(TemplateToken::Text(rest[..start].to_string()));
      }
      if let Some(end) = rest.find("]") {
        let var = rest[start + 2..end].trim().to_string();
        if var.starts_with("for ") {
          let parts: Vec<&str> = var.split_whitespace().collect();
          if parts.len() == 3 {
            tokens.push(TemplateToken::For(parts[1].to_string(), parts[2].to_string()));
          }
        } else if var.starts_with("endfor ") {
          tokens.push(TemplateToken::EndFor(var[7..].trim().to_string()));
        } else if var.starts_with("if ") {
          tokens.push(TemplateToken::If(var[3..].trim().to_string()));
        } else if var == "endif" {
          tokens.push(TemplateToken::EndIf);
        } else {
          tokens.push(TemplateToken::Var(var));
        }
        rest = &rest[end + 2..];
      } else {
        break; // Malformed template
      }
    } else {
      // All remaining is text
      tokens.push(TemplateToken::Text(rest.to_string()));
      break;
    }
  }

 */
fn tokenize_content(input: &str) -> Result<Vec<TemplateToken>, String> {
  let mut tokens = Vec::new();
  let mut rest = input;
  while !rest.is_empty() {
    if let Some(from) = rest.find('[') {
      if from > 0 {
        let text = rest[..from].to_string();
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
      let text = rest.to_string();
      let text = TemplateToken::Text(text);
      println!("text: {:?}", text);
      tokens.push(text);
      break;
    }
  }
  Ok(tokens)
}

fn parse_tag(input: &str) -> Result<TemplateToken, String> {
  let parts: Vec<&str> = input.split_whitespace().collect();
  let tag =
    if parts[0] == "for" {
      parse_for_tag(parts)?
    }
    else if parts[0] == "endfor" {
      TemplateToken::EndFor(input[7..].trim().to_string())
    }
    else {
      TemplateToken::Var(input.to_string())
    };
  Ok(tag)
}

fn parse_for_tag(parts: Vec<&str>) -> Result<TemplateToken, String> {
  assert!(parts[0] == "for", "Expected 'for' tag, got: {}", parts[0]);
  if parts.len() == 4 {
    if parts[2] == "in" {
      Ok(TemplateToken::For(parts[1].to_string(), parts[3].to_string()))
    }
    else {
      Err("Invalid for tag syntax. Missing 'in' keyword.".to_string())
    }
  } else {
    Err(
      format!(
        "Invalid for tag syntax. Incorrect number of parts - expected 4 (for, var, in, expression), got {:?}.",
        parts
      )
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tokenize_content_handles_text() {
    let result = tokenize_content("Hello, world!").unwrap();
    assert_eq!(
      vec![TemplateToken::Text("Hello, world!".to_string())],
      result
    );
  }

  #[test]
  fn tokenize_content_handles_var() {
    let result = tokenize_content("[section.title]").unwrap();
    assert_eq!(
      vec![TemplateToken::Var("section.title".to_string())],
      result
    );
  }

  #[test]
  fn tokenize_content_fails_if_no_closing_bracket() {
    let result = tokenize_content("[section.title").unwrap_err();
    assert!(result.to_string().contains("Missing closing bracket."));
  }

  #[test]
  fn tokenize_content_handles_mixed_text_and_var() {
    let result = tokenize_content("Hello, [section.title]!").unwrap();
    assert_eq!(
      vec![
        TemplateToken::Text("Hello, ".to_string()),
        TemplateToken::Var("section.title".to_string()),
        TemplateToken::Text("!".to_string())
      ],
      result
    );
  }

  #[test]
  fn tokenize_content_handles_foreach() {
    let result = tokenize_content("[ for content in section.content ]\nSome text.\n[ endfor content ]").unwrap();
    assert_eq!(
      vec![
        TemplateToken::For("content".to_string(), "section.content".to_string()),
        TemplateToken::Text("\nSome text.\n".to_string()),
        TemplateToken::EndFor("content".to_string())
      ],
      result
    );
  }

  #[test]
  fn tokenize_content_fails_if_foreach_syntax_is_invalid() {
    let error = "Invalid for tag syntax.";
    assert_invalid_syntax("[ for ]", error);
    assert_invalid_syntax("[ for section in ]", error);
    assert_invalid_syntax("[ for in section.content ]", error);
    assert_invalid_syntax("[ for content section.content ]", error);
    assert_invalid_syntax("[ for content : section.content ]", error);
  }

  fn assert_invalid_syntax(input: &str, expected: &str) {
    let err = super::tokenize_content(input).unwrap_err();
    assert!(err.contains(expected),
      "Expected error for input '{}', got: {}", input, err);
  }
}
