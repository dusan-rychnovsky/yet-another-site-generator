use config::{Config, Mode};

pub mod config;
pub mod data_file_parser;
pub mod expressions;
pub mod template_parser;
pub mod template_tokenizer;
pub mod visitor;

/// Populates HTML template files with values from YAML data files.
///
/// Highlevel architecture of the library:
///
/// ```mermaid
/// flowchart LR
///   DataFile["Data File
///   *YAML*"]
///   DataFileParser>"data_file_parser::parse"]
///   TemplateFile["Template File
///   *HTML + custom tags*"]
///   TemplateTokenizer>"template_tokenizer::tokenize"]
///   TemplateTokens["TemplateToken[]
///   *-- lexical tokens*"]
///   TemplateParser>"template_parser::parse"]
///   TemplateTree["TemplateTree
///   *-- syntax tree*"]
///   DataSet
///   Visitor>"visitor::visit"]
///   PopulatedFile["Populated File
///   *HTML*"]
///
///   DataFile --> DataFileParser --> DataSet
///   TemplateFile --> TemplateTokenizer --> TemplateTokens --> TemplateParser --> TemplateTree
///   DataSet --> Visitor --> PopulatedFile
///   TemplateTree --> Visitor
///
///   classDef border-dashed stroke-dasharray: 5 5
///   class DataFileParser,TemplateTokenizer,TemplateParser,Visitor border-dashed
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let config = Config::parse(&args);
    match config {
        Ok(Config {
            mode:
                Mode::SingleFile {
                    data_file_path,
                    template_file_path,
                },
        }) => {
            let result = yasg::populate_file(data_file_path, Option::Some(template_file_path))?;
            println!("{}", result);
        }
        Ok(Config {
            mode:
                Mode::Recursive {
                    src_dir_path,
                    dst_dir_path,
                },
        }) => {
            yasg::populate_all_files(src_dir_path, dst_dir_path)?;
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!("{}", config::print_usage(&args));
            std::process::exit(1);
        }
    }

    Ok(())
}
