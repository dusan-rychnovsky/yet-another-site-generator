use crate::template_parser::{self, TemplateTree};
use crate::template_tokenizer::{self, TemplateToken};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

struct CacheEntry {
    tokens: Vec<TemplateToken>,
    tree: TemplateTree,
}

pub struct TemplateCache {
    items: HashMap<String, CacheEntry>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn load_template_tree(
        &mut self,
        template_file_path: &str,
    ) -> Result<&TemplateTree, String> {
        Ok(&self.load_template(template_file_path)?.tree)
    }

    fn load_template(&mut self, template_file_path: &str) -> Result<&CacheEntry, String> {
        let cache_key = normalize_path(Path::new(template_file_path))
            .to_string_lossy()
            .into_owned();
        if !self.items.contains_key(&cache_key) {
            let file_content = fs::read_to_string(template_file_path).map_err(|e| {
                format!(
                    "Failed to read template file content. File: '{template_file_path}'. Error: '{e}'."
                )
            })?;
            let tokens = template_tokenizer::tokenize(&file_content).map_err(|e| {
                format!(
                    "Failed to tokenize template file content. File: '{template_file_path}'. Error: '{e}'."
                )
            })?;
            let mut final_tokens = Vec::new();
            for token in tokens {
                match token {
                    TemplateToken::Include(file_path) => {
                        let file_path = Path::new(template_file_path)
                            .parent()
                            .unwrap_or_else(|| Path::new(""))
                            .join(file_path);
                        let included = self.load_template(&file_path.to_string_lossy())?;
                        final_tokens.extend(included.tokens.iter().cloned());
                    }
                    other => final_tokens.push(other),
                }
            }
            let tree = template_parser::parse(&final_tokens).map_err(|e| {
                format!(
                    "Failed to parse template file content. File: '{template_file_path}'. Error: '{e}'."
                )
            })?;
            self.items.insert(
                cache_key.clone(),
                CacheEntry {
                    tokens: final_tokens,
                    tree,
                },
            );
        }
        Ok(&self.items[&cache_key])
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::Prefix(_) | Component::RootDir) => {}
                _ => normalized.push(".."),
            },
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_keeps_already_normalized_path() {
        assert_eq!(
            PathBuf::from("tests/data/blog/index.html"),
            normalize_path(Path::new("tests/data/blog/index.html"))
        );
    }

    #[test]
    fn normalize_path_removes_inner_current_dir() {
        assert_eq!(
            PathBuf::from("tests/data/blog/index.html"),
            normalize_path(Path::new("tests/data/blog/./index.html"))
        );
    }

    #[test]
    fn normalize_path_removes_leading_current_dir() {
        assert_eq!(PathBuf::from("a/b"), normalize_path(Path::new("./a/b")));
    }

    #[test]
    fn normalize_path_resolves_parent_dir() {
        assert_eq!(PathBuf::from("b"), normalize_path(Path::new("a/../b")));
    }

    #[test]
    fn normalize_path_resolves_multiple_parent_dirs() {
        assert_eq!(
            PathBuf::from("tests/data/blog/snippets/menu.html"),
            normalize_path(Path::new(
                "tests/data/blog/cooking/recipes/../../snippets/menu.html"
            ))
        );
    }

    #[test]
    fn normalize_path_collapses_to_empty_when_all_cancels() {
        assert_eq!(PathBuf::new(), normalize_path(Path::new("a/..")));
    }

    #[test]
    fn normalize_path_keeps_leading_parent_dirs() {
        assert_eq!(
            PathBuf::from("../../a"),
            normalize_path(Path::new("../../a"))
        );
    }

    #[test]
    fn normalize_path_keeps_unresolvable_parent_dir() {
        assert_eq!(
            PathBuf::from("../a"),
            normalize_path(Path::new("b/../../a"))
        );
    }

    #[test]
    fn normalize_path_handles_empty_path() {
        assert_eq!(PathBuf::new(), normalize_path(Path::new("")));
    }

    #[test]
    fn normalize_path_maps_different_spellings_to_same_key() {
        let from_index = normalize_path(Path::new("tests/data/blog/./snippets/menu.html"));
        let from_recipe = normalize_path(Path::new(
            "tests/data/blog/cooking/recipes/../../snippets/menu.html",
        ));
        assert_eq!(from_index, from_recipe);
    }

    #[cfg(windows)]
    #[test]
    fn normalize_path_unifies_mixed_separators_on_windows() {
        assert_eq!(
            normalize_path(Path::new("tests/data/blog/./snippets/menu.html")),
            normalize_path(Path::new(
                "tests/data/blog\\cooking\\recipes\\..\\..\\snippets/menu.html"
            ))
        );
    }
}
