pub mod data_file_parser;
pub mod template_parser;

fn main() {
  data_file_parser::parse("tests/data/data-go.yml");;
}
