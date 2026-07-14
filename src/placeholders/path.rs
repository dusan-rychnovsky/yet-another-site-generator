use crate::data_file_parser::Node;
use std::path::Path;

/// Embeds the `PATH` placeholder — the filesystem path of the page's data file — into the given
/// page node. Non-map nodes are left unchanged.
pub fn embed(page: &mut Node, path: &Path) {
    if let Node::Map(map) = page {
        map.insert(
            "PATH".to_string(),
            Node::Str(path.to_string_lossy().into_owned()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_adds_the_file_path_to_a_page_map() {
        let value: serde_yaml::Value = serde_yaml::from_str("title: Hello").unwrap();
        let mut page = Node::from_yaml(&value);

        embed(&mut page, Path::new("blog/index.yml"));

        match page {
            Node::Map(map) => match map.get("PATH") {
                Some(Node::Str(path)) => assert_eq!(path.as_str(), "blog/index.yml"),
                other => panic!("expected a PATH string, got {other:?}"),
            },
            other => panic!("expected a map, got {other:?}"),
        }
    }
}
