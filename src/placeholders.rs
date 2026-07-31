use crate::data_file_parser::Node;
use std::rc::Rc;

pub mod categories;

pub fn embed(data_set_trees_with_paths: &mut [(String, Rc<Node>)]) -> Result<(), String> {
    // insert individual placeholders
    for (file_path, tree) in data_set_trees_with_paths.iter_mut() {
        if let Node::Map(map) = Rc::make_mut(tree) {
            map.insert(
                "PATH".to_string(),
                Rc::new(Node::Str(file_path.to_string())),
            );
        } else {
            return Err(format!(
                "Expected a map at the root of data file '{}', but found {:?}",
                file_path, tree
            ));
        }
    }

    // insert group placeholders
    let data_set_trees = data_set_trees_with_paths
        .iter()
        .map(|(_, tree)| Rc::clone(tree))
        .collect::<Vec<_>>();
    let pages = Rc::new(Node::Seq(data_set_trees.clone()));
    let categories = categories::build(&data_set_trees);
    for (file_path, tree) in data_set_trees_with_paths.iter_mut() {
        if let Node::Map(map) = Rc::make_mut(tree) {
            map.insert("PAGES".to_string(), Rc::clone(&pages));
            map.insert("CATEGORIES".to_string(), Rc::clone(&categories));
        } else {
            return Err(format!(
                "Expected a map at the root of data file '{}', but found {:?}",
                file_path, tree
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(node: &Node) -> Vec<&str> {
        match node {
            Node::Map(map) => {
                let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
                keys.sort();
                keys
            }
            other => panic!("expected a map, got {other:?}"),
        }
    }

    #[test]
    fn embed_adds_path_pages_and_categories_to_every_render_root() {
        let mut data = vec![
            (
                "blog/a.yml".to_string(),
                Node::from_yaml_text("title: A").unwrap(),
            ),
            (
                "blog/b.yml".to_string(),
                Node::from_yaml_text("title: B").unwrap(),
            ),
        ];

        embed(&mut data).unwrap();

        assert_eq!(
            keys(&data[0].1),
            vec!["CATEGORIES", "PAGES", "PATH", "title"]
        );
        assert_eq!(
            data[0].1.get_map_child("PATH").unwrap().get_text().unwrap(),
            "blog/a.yml"
        );
        assert_eq!(
            keys(&data[1].1),
            vec!["CATEGORIES", "PAGES", "PATH", "title"]
        );
        assert_eq!(
            data[1].1.get_map_child("PATH").unwrap().get_text().unwrap(),
            "blog/b.yml"
        );
    }

    #[test]
    fn embed_lists_every_page_with_its_path_in_pages() {
        let mut data = vec![
            (
                "blog/a.yml".to_string(),
                Node::from_yaml_text("title: A").unwrap(),
            ),
            (
                "blog/b.yml".to_string(),
                Node::from_yaml_text("title: B").unwrap(),
            ),
            (
                "blog/c.yml".to_string(),
                Node::from_yaml_text("title: C").unwrap(),
            ),
        ];

        embed(&mut data).unwrap();

        for (_, root) in &data {
            let pages = root.get_map_child("PAGES").unwrap().get_seq().unwrap();
            let paths: Vec<&str> = pages
                .iter()
                .map(|page| page.get_map_child("PATH").unwrap().get_text().unwrap())
                .collect();
            assert_eq!(paths, vec!["blog/a.yml", "blog/b.yml", "blog/c.yml"]);
            let titles: Vec<&str> = pages
                .iter()
                .map(|page| page.get_map_child("title").unwrap().get_text().unwrap())
                .collect();
            assert_eq!(titles, vec!["A", "B", "C"]);
        }
    }

    #[test]
    fn embed_includes_path_in_pages_within_categories() {
        let mut data = vec![(
            "blog/a.yml".to_string(),
            Node::from_yaml_text("title: A\ncategories: [home, blog]").unwrap(),
        )];

        embed(&mut data).unwrap();

        let categories = data[0]
            .1
            .get_map_child("CATEGORIES")
            .unwrap()
            .get_seq()
            .unwrap();
        let home = &categories[0];
        let blog = &home
            .get_map_child("subcategories")
            .unwrap()
            .get_seq()
            .unwrap()[0];
        let pages = blog.get_map_child("pages").unwrap().get_seq().unwrap();
        assert_eq!(
            pages[0].get_map_child("PATH").unwrap().get_text().unwrap(),
            "blog/a.yml"
        );
    }

    #[test]
    fn embed_errors_when_a_root_is_not_a_map() {
        let mut data = vec![(
            "scalar.yml".to_string(),
            Node::from_yaml_text("just a scalar").unwrap(),
        )];

        let error = embed(&mut data).unwrap_err();

        assert_eq!(
            error,
            "Expected a map at the root of data file 'scalar.yml', but found Str(\"just a scalar\")"
        );
    }
}
