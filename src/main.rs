pub mod data_file_parser;
pub mod template_parser;
pub mod template_tokenizer;
pub mod expressions;

fn main() {
  data_file_parser::parse("tests/data/data-go.yml");
}
