use yet_another_site_generator::template_tokenizer::{self, TemplateToken};

#[test]
fn tokenize_template_go_html() {
    let tokens = template_tokenizer::tokenize("tests/data/template-go.html");

    assert!(tokens.is_ok(), "Expected to tokenize template file successfully. Error: {:?}", tokens.err());
    let tokens = tokens.unwrap();

    assert_eq!(53, tokens.len());

    assert_token_text(
      &tokens[0],
      "<!DOCTYPE html>\n<html lang=\"cs\">",
      "<title>Mé oblíbené zdroje informací na téma: "
    );
    assert_eq!(TemplateToken::Var(String::from("title")), tokens[1]);
    assert_token_text(
      &tokens[2],
      "</title>\n  <link ",
      "<ol class=\"breadcrumb mb-0\">\n        "
    );
    assert_eq!(TemplateToken::For(String::from("crumb"), String::from("crumbs")), tokens[3]);
    assert_eq!(TemplateToken::Text(String::from("\n          <li class=\"breadcrumb-item\"><a href=\"")), tokens[4]);
    assert_eq!(TemplateToken::Var(String::from("crumb.href")), tokens[5]);
    assert_eq!(TemplateToken::Text(String::from("\">")), tokens[6]);
    assert_eq!(TemplateToken::Var(String::from("crumb.text")), tokens[7]);
    assert_eq!(TemplateToken::Text(String::from("</a></li>\n        ")), tokens[8]);
    assert_eq!(TemplateToken::EndFor(String::from("crumbs")), tokens[9]);
    // ...
    assert_eq!(TemplateToken::If("exists section.subsections".to_string()), tokens[29]);
    assert_eq!(TemplateToken::Text("\n                  <ul>\n                    ".to_string()), tokens[30]);
    assert_eq!(TemplateToken::For("subsection".to_string(), "section.subsections".to_string()), tokens[31]);
    assert_eq!(TemplateToken::Text("\n                      <li class=\"mb-1\">\n                        <em>".to_string()), tokens[32]);
    assert_eq!(TemplateToken::Var("subsection.title".to_string()), tokens[33]);
    assert_eq!(TemplateToken::Text("</em> - ".to_string()), tokens[34]);
    assert_eq!(TemplateToken::Var("subsection.content".to_string()), tokens[35]);
    assert_eq!(TemplateToken::Text("\n                      </li>\n                    ".to_string()), tokens[36]);
    assert_eq!(TemplateToken::EndFor("subsection".to_string()), tokens[37]);
    assert_eq!(TemplateToken::Text("\n                  </ul>\n                ".to_string()), tokens[38]);
    assert_eq!(TemplateToken::EndIf, tokens[39]);
    // ...
    assert_token_text(
      &tokens[52],
      "\n      </div>\n    </div>",
      "</body>\n</html>\n"
    );
}

fn assert_token_text(token: &TemplateToken, expected_start: &str, expected_end: &str) {
  match token {
    TemplateToken::Text(s) => {
      assert!(s.starts_with(expected_start), "unexpected text: '{}'", s);
      assert!(s.ends_with(expected_end), "unexpected text: '{}'", s);
    }
    other => panic!("unexpected token: {:?}", other),
  }
}