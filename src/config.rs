#[derive(Debug)]
pub struct Config<'a> {
    pub mode: Mode<'a>,
}

#[derive(Debug)]
pub enum Mode<'a> {
    SingleFile {
        data_file_path: &'a str,
        template_file_path: &'a str,
    },
    Recursive {
        src_dir_path: &'a str,
        dst_dir_path: &'a str,
    },
}

/// Generates a usage message that describes how to use the command line interface,
/// which should be printed to the console.
pub fn print_usage(args: &[String]) -> String {
    format!(
        "\
Usage: {} <data-file> <template-file>
   or: {} -r <source-dir> <dest-dir>",
        args[0], args[0]
    )
}

impl<'a> Config<'a> {
    /// Parses command line arguments and returns a [`Config`] instance, or an error
    /// if the arguments are invalid, such as if the mode is not recognized
    /// or if not enough arguments are provided.
    pub fn parse(args: &'a [String]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments provided.");
        }

        if args[1] == "-r" {
            if args.len() != 4 {
                Err("Recursive mode requires two parameters.")
            } else {
                Ok(Config {
                    mode: Mode::Recursive {
                        src_dir_path: &args[2],
                        dst_dir_path: &args[3],
                    },
                })
            }
        } else if args.len() != 3 {
            Err("Single-file mode requires two parameters.")
        } else {
            Ok(Config {
                mode: Mode::SingleFile {
                    data_file_path: &args[1],
                    template_file_path: &args[2],
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_supports_single_file_mode() {
        let args = vec![
            "yasg".to_string(),
            "data.yaml".to_string(),
            "template.html".to_string(),
        ];
        let config =
            Config::parse(&args).unwrap_or_else(|e| panic!("Failed to parse config: {}", e));
        match config.mode {
            Mode::SingleFile {
                data_file_path,
                template_file_path,
            } => {
                assert_eq!(data_file_path, "data.yaml");
                assert_eq!(template_file_path, "template.html");
            }
            _ => panic!("Expected SingleFile mode, got {:#?}", config),
        }
    }

    #[test]
    fn parse_single_file_mode_fails_if_not_enough_args() {
        let args = vec!["yasg".to_string(), "data.yaml".to_string()];
        let config = Config::parse(&args);
        let err = config.unwrap_err();
        assert_eq!(err, "Single-file mode requires two parameters.");
    }

    #[test]
    fn parse_single_file_mode_fails_if_too_many_args() {
        let args = vec![
            "yasg".to_string(),
            "data1.yaml".to_string(),
            "data2.yaml".to_string(),
            "template.html".to_string(),
        ];
        let config = Config::parse(&args);
        let err = config.unwrap_err();
        assert_eq!(err, "Single-file mode requires two parameters.");
    }

    #[test]
    fn parse_supports_recursive_mode() {
        let args = vec![
            "yasg".to_string(),
            "-r".to_string(),
            "src".to_string(),
            "dst".to_string(),
        ];
        let config =
            Config::parse(&args).unwrap_or_else(|e| panic!("Failed to parse config: {}", e));
        match config.mode {
            Mode::Recursive {
                src_dir_path,
                dst_dir_path,
            } => {
                assert_eq!(src_dir_path, "src");
                assert_eq!(dst_dir_path, "dst");
            }
            _ => panic!("Expected Recursive mode, got {:#?}", config),
        }
    }

    #[test]
    fn parse_recursive_mode_fails_if_not_enough_args() {
        let args = vec!["yasg".to_string(), "-r".to_string(), "src".to_string()];
        let config = Config::parse(&args);
        let err = config.unwrap_err();
        assert_eq!(err, "Recursive mode requires two parameters.");
    }

    #[test]
    fn parse_recursive_mode_fails_if_too_many_args() {
        let args = vec![
            "yasg".to_string(),
            "-r".to_string(),
            "src".to_string(),
            "dst".to_string(),
            "extra".to_string(),
        ];
        let config = Config::parse(&args);
        let err = config.unwrap_err();
        assert_eq!(err, "Recursive mode requires two parameters.");
    }
}
