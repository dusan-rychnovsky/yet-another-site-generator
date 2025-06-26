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
fn tokenize_content(input: &str) -> Result<Vec<TemplateToken>, Box<dyn std::error::Error>> {
  let mut tokens = Vec::new();

  let mut rest = input;
  while !rest.is_empty() {
    if let Some(start) = rest.find('[') {
      let expr = &rest[start+1..rest.len()-1];
      tokens.push(TemplateToken::Var(expr.to_string()));
      break;
    }
    else {
      tokens.push(TemplateToken::Text(rest.to_string()));
      break;
    }
  }

  Ok(tokens)
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
}
