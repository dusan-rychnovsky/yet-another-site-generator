use std::fs;
use yasg::data_file_parser;
use yasg::data_file_parser::DataSet;
use yasg::expressions::Path;

#[test]
fn parse_loads_and_parses_data_file() {
    let content = fs::read_to_string("tests/data/example-data.yml")
        .unwrap_or_else(|e| panic!("Failed to read data file: {}", e));
    let data = data_file_parser::parse(&content)
        .unwrap_or_else(|e| panic!("Failed to parse data file: {}", e));
    let data_set = DataSet::from(&data);

    assert_eq!(
        data_set.get_str(&Path::parse("title")).unwrap(),
        "Hello World!"
    );

    let items = data_set.list("", &Path::parse("backpack.items")).unwrap();
    assert_eq!(items.len(), 3);

    assert_eq!(
        items[0].get_str(&Path::parse("name")).unwrap(),
        "sleeping bag"
    );
    assert_eq!(items[0].get_str(&Path::parse("weight")).unwrap(), "1.5kg");

    assert_eq!(items[1].get_str(&Path::parse("name")).unwrap(), "tent");
    assert_eq!(items[1].get_str(&Path::parse("weight")).unwrap(), "2.0kg");

    assert_eq!(
        items[2].get_str(&Path::parse("name")).unwrap(),
        "water bottle"
    );
    assert_eq!(items[2].get_str(&Path::parse("weight")).unwrap(), "0.5kg");
}
