use data_file_parser::DataSet;
use data_file_parser::Node;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use template_cache::TemplateCache;
use walkdir::WalkDir;

pub mod data_file_parser;
pub mod expressions;
pub mod placeholders;
pub mod template_cache;
pub mod template_parser;
pub mod template_tokenizer;
pub mod visitor;

/// Looks up all data files in the given source directory. For each data file, loads the linked template file
/// and populates it. The populated files are saved in the given destination directory, mirroring the data file
/// paths.
pub fn populate_all_files(
    src_dir_path: &str,
    dst_dir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    check_dir_exists(src_dir_path)?;
    check_dir_exists(dst_dir_path)?;

    let mut template_cache = TemplateCache::new();
    for (data_file_path, value) in &load_yamls(src_dir_path)? {
        let root = Node::from_yaml(value);
        let data_set = DataSet::from(&root);

        let populated_content = populate_data_set(
            &data_set,
            data_file_path.to_str().unwrap(),
            None,
            &mut template_cache,
        )?;
        let (output_path, output_dir_path) =
            construct_output_path(data_file_path, src_dir_path, dst_dir_path)?;

        fs::create_dir_all(output_dir_path)?;
        fs::write(&output_path, populated_content)?;

        println!("Generated: {:?}", output_path);
    }
    Ok(())
}

/// Looks up all data files in the given source directory recursively. Like recursive mode, each
/// data file is populated and saved in the destination directory mirroring its source path.
/// Additionally, the following virtual placeholders are made available for every page:
/// - PAGES - a sequence of datasources of all pages of the blog.
/// - CATEGORIES - a tree of datasources of all pages groupped by categories chains.
/// - PATH - embedded in every page's datasource; the filesystem path of the page's data file.
pub fn populate_blog(
    src_dir_path: &str,
    dst_dir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    check_dir_exists(src_dir_path)?;
    check_dir_exists(dst_dir_path)?;

    let pages = load_yamls(src_dir_path)?;
    let page_nodes: Vec<Node> = pages
        .iter()
        .map(|(file_path, value)| placeholders::path::embed(Node::from_yaml(value), file_path))
        .collect();
    let pages_placeholder = placeholders::pages::build(&page_nodes);
    let categories_placeholder = placeholders::categories::build(&page_nodes);

    let mut template_cache = TemplateCache::new();
    for ((data_file_path, _), page_node) in pages.iter().zip(&page_nodes) {
        let root = placeholders::insert_virtual_placeholders(
            page_node,
            &pages_placeholder,
            &categories_placeholder,
        );
        let data_set = DataSet::from(&root);

        let populated_content = populate_data_set(
            &data_set,
            data_file_path.to_str().unwrap(),
            None,
            &mut template_cache,
        )?;
        let (output_path, output_dir_path) =
            construct_output_path(data_file_path, src_dir_path, dst_dir_path)?;

        fs::create_dir_all(output_dir_path)?;
        fs::write(&output_path, populated_content)?;

        println!("Generated: {:?}", output_path);
    }

    Ok(())
}

/// Collects and parses all dataset files in the given source directory recursively.
fn load_yamls(dir_path: &str) -> Result<Vec<(PathBuf, serde_yaml::Value)>, String> {
    let mut yamls: Vec<(PathBuf, serde_yaml::Value)> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "yml"))
        .map(|e| e.path().to_path_buf())
        .map(|p| {
            let content = fs::read_to_string(&p).map_err(|e| {
                format!(
                    "Failed to read data file content. File: '{}'. Error: '{}'.",
                    p.display(),
                    e
                )
            })?;
            let serde = data_file_parser::parse(&content).map_err(|e| {
                format!(
                    "Failed to parse data file content. File: '{}'. Error: '{}'.",
                    p.display(),
                    e
                )
            })?;
            Ok::<_, String>((p, serde))
        })
        .collect::<Result<_, _>>()?;
    yamls.sort_by(|(a, _), (b, _)| a.cmp(b));
    Ok(yamls)
}

/// Checks that the given path exists and represents a directory.
fn check_dir_exists(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!(
            "Failed to load directory. Dir: '{}'. Error: 'Path does not exist.'.",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!(
            "Failed to load directory. Dir: '{}'. Error: 'Path is not a directory.'.",
            path.display()
        ));
    }
    Ok(())
}

/// Returns file path in the given destination directory, which should contain the output of processing the given data file.
/// The data file is expected to be located in the given source directory.
/// Note that populated files are placed in the same relative locations as source data files.
fn construct_output_path(
    data_file_path: &Path,
    src_dir_path: &str,
    dst_dir_path: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let relative_data_file_path = data_file_path
        .strip_prefix(src_dir_path)
        .map_err(|e| format!("Failed to resolve relative data file path. Error: '{}'.", e))?;
    let output_path = Path::new(dst_dir_path)
        .join(relative_data_file_path)
        .with_extension("html");
    let output_dir_path = output_path
        .parent()
        .ok_or_else(|| {
            format!(
                "Failed to resolve parent directory path. File path: '{}'.",
                output_path.display()
            )
        })?
        .to_path_buf();
    Ok((output_path, output_dir_path))
}

/// Populates the given template file using the given data file and returns the populated file content.
pub fn populate_file(
    data_file_path: &str,
    template_file_path: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let data_content = fs::read_to_string(data_file_path).map_err(|e| {
        format!(
            "Failed to read data file content. File: '{}'. Error: '{}'.",
            data_file_path, e
        )
    })?;
    let data = data_file_parser::parse(&data_content).map_err(|e| {
        format!(
            "Failed to parse data file content. File: '{}'. Error: '{}'.",
            data_file_path, e
        )
    })?;
    let root = Node::from_yaml(&data);
    let data_set = DataSet::from(&root);
    let mut template_cache = TemplateCache::new();
    populate_data_set(
        &data_set,
        data_file_path,
        template_file_path,
        &mut template_cache,
    )
}

/// Populates the template linked to the given data set and returns the populated content.
/// The template is looked up via [`look_up_template_file_path`].
fn populate_data_set(
    data_set: &DataSet,
    data_file_path: &str,
    template_file_path: Option<&str>,
    template_cache: &mut TemplateCache,
) -> Result<String, Box<dyn std::error::Error>> {
    let template_file_path =
        look_up_template_file_path(data_set, data_file_path, template_file_path)?;
    let template_tree = template_cache
        .load_template(&template_file_path)
        .map_err(|e| {
            format!(
                "Failed to populate data file. File: '{}'. {}",
                data_file_path, e
            )
        })?;

    let result = visitor::visit(template_tree, data_set).map_err(|e| {
        format!(
            "Failed to populate data file. File: '{}'. Error: '{}'.",
            data_file_path, e
        )
    })?;

    Ok(result)
}

/// Resolves the template file to be used with the given data file.
/// If a template file path is explicitly provided, it will be used. Otherwise,
/// the path is looked up from the `template` field in the given data set.
fn look_up_template_file_path(
    data_set: &DataSet,
    data_file_path: &str,
    template_file_path: Option<&str>,
) -> Result<String, String> {
    let template_file_path = if let Some(template_file_path) = template_file_path {
        template_file_path.to_string()
    } else {
        let template_file_path = data_set
            .get_str(&expressions::Path::from_segment("template"))
            .and_then(|v| {
                v.ok_or_else(|| "Path [template] is not defined in data file.".to_string())
            })
            .map_err(|e| {
                format!(
                    "Failed to parse data file content. File: '{}'. Error: '{}'.",
                    data_file_path, e
                )
            })?;
        let parent_path = Path::new(data_file_path).parent().unwrap();
        parent_path
            .join(template_file_path)
            .to_string_lossy()
            .to_string()
    };
    Ok(template_file_path)
}
