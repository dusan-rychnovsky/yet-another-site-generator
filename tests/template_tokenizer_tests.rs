use yet_another_site_generator::template_tokenizer::{self, TemplateToken};

#[test]
fn tokenize_template_go_html() {
    let tokens = template_tokenizer::tokenize("tests/data/template-go.html");

    assert!(tokens.is_ok(), "Expected to tokenize template file successfully. Error: {:?}", tokens.err());
    let tokens = tokens.unwrap();

    assert_eq!(44, tokens.len());

    match &tokens[0] {
      TemplateToken::Text(s) => {
        assert!(s.starts_with("<!DOCTYPE html>\n<html lang=\"cs\">"));
      }
      other => panic!("tokens[0] is not TemplateToken::Text: {:?}", other),
    }

    assert_eq!(TemplateToken::Var(String::from("title")), tokens[1]);
}