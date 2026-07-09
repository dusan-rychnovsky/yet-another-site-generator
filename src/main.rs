use config::{Config, Mode};
use log::{error, info};

pub mod config;
pub mod data_file_parser;
pub mod expressions;
pub mod template_parser;
pub mod template_tokenizer;
pub mod visitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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
            info!(
                "MODE: SingleFile. Data File: '{}'. Template File: '{}'.",
                data_file_path, template_file_path
            );
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
            info!(
                "MODE: Recursive. Source Dir: '{}'. Dest Dir: '{}'.",
                src_dir_path, dst_dir_path
            );
            yasg::populate_all_files(src_dir_path, dst_dir_path)?;
        }
        Ok(Config {
            mode:
                Mode::Blog {
                    src_dir_path,
                    dst_dir_path,
                },
        }) => {
            info!(
                "MODE: Blog. Source Dir: '{}'. Dest Dir: '{}'.",
                src_dir_path, dst_dir_path
            );
            yasg::populate_blog(src_dir_path, dst_dir_path)?;
        }
        Err(err) => {
            error!("Error: {}", err);
            eprintln!("{}", config::print_usage(&args));
            std::process::exit(1);
        }
    }

    Ok(())
}
