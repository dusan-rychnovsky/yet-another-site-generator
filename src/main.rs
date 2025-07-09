use config::{Config, Mode};

pub mod config;
pub mod data_file_parser;
pub mod template_parser;
pub mod template_tokenizer;
pub mod expressions;
pub mod visitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();

  let config = Config::parse(&args);
  match config {
    Ok(Config { mode: Mode::SingleFile { data_path, template_path } }) => {
      yasg::process_single_file(data_path, template_path)?;
    }
    Ok(Config { mode: Mode::Recursive { src_root_path, dst_root_path } }) => {
      yasg::process_recursive(src_root_path, dst_root_path)?;
    }
    Err(err) => {
      eprintln!("Error: {}", err);
      eprintln!("{}", config::print_usage(&args));
      std::process::exit(1);
    }
  }

  Ok(())
}
