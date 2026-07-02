use crate::template_parser::{self, TemplateTree};
use crate::template_tokenizer;
use self_cell::self_cell;
use std::collections::HashMap;
use std::fs;

self_cell!(
    struct CacheEntry {
        owner: String,

        #[covariant]
        dependent: TemplateTree,
    }
);

pub struct TemplateCache {
    items: HashMap<String, CacheEntry>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn load_template(&mut self, template_file_path: &str) -> Result<&TemplateTree<'_>, String> {
        if !self.items.contains_key(template_file_path) {
            let template_file_content = fs::read_to_string(template_file_path).map_err(|e| {
                format!(
                    "Failed to read template file content. File: '{template_file_path}'. Error: '{e}'."
                )
            })?;

            let entry = CacheEntry::try_new(template_file_content, |content| {
                let tokens = template_tokenizer::tokenize(content).map_err(|e| {
                    format!(
                        "Failed to tokenize template file content. File: '{template_file_path}'. Error: '{e}'."
                    )
                })?;
                template_parser::parse(&tokens).map_err(|e| {
                    format!(
                        "Failed to parse template file content. File: '{template_file_path}'. Error: '{e}'."
                    )
                })
            })?;

            self.items.insert(template_file_path.to_string(), entry);
        }

        Ok(self
            .items
            .get(template_file_path)
            .expect("The value has just been inserted.")
            .borrow_dependent())
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}
