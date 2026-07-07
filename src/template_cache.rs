use crate::template_parser::{self, TemplateTree};
use crate::template_tokenizer;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs;

pub struct TemplateCache {
    items: HashMap<String, TemplateTree>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn load_template(&mut self, template_file_path: &str) -> Result<&TemplateTree, String> {
        match self.items.entry(template_file_path.to_string()) {
            Occupied(entry) => Ok(entry.into_mut()),
            Vacant(entry) => {
                let template_file_content = fs::read_to_string(template_file_path).map_err(|e| {
                    format!(
                        "Failed to read template file content. File: '{template_file_path}'. Error: '{e}'."
                    )
                })?;
                let tokens = template_tokenizer::tokenize(&template_file_content).map_err(|e| {
                    format!(
                        "Failed to tokenize template file content. File: '{template_file_path}'. Error: '{e}'."
                    )
                })?;
                let tree = template_parser::parse(&tokens).map_err(|e| {
                    format!(
                        "Failed to parse template file content. File: '{template_file_path}'. Error: '{e}'."
                    )
                })?;
                Ok(entry.insert(tree))
            }
        }
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}
