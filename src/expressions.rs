#[derive(Debug, PartialEq, Clone)]
pub struct Path<'a> {
  pub segments: Vec<&'a str>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr<'a> {
  pub predicate: Predicate,
  pub path: Path<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Predicate {
  Exists
}

impl<'a> Path<'a> {
  pub fn parse(input: &'a str) -> Self {
    Path {
      segments: input.split('.').collect()
    }
  }

  pub fn from(segments: Vec<&'a str>) -> Self {
    Path { segments }
  }
}

impl<'a> Expr<'a> {
  pub fn parse(parts: Vec<&'a str>) -> Result<Self, String> {
    if parts.len() != 2 {
      return Err(format!("Invalid expression syntax - expected a predicate and a path, got: '{:#?}'.", parts));
    }
    let predicate = match parts[0] {
      "exists" => Predicate::Exists,
      _ => return Err(format!("Unknown predicate: '{}'.", parts[0])),
    };
    let path = Path::parse(parts[1]);
    Ok(Expr {
      predicate,
      path,
    })
  }

  pub fn from(predicate: Predicate, segments: Vec<&'a str>) -> Self {
    Expr {
      predicate: predicate,
      path: Path::from(segments)
    }
  }
}
