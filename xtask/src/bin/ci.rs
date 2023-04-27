use markdown::mdast::{Definition, Heading};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::string::String;

use cargo_metadata::{CargoOpt, MetadataCommand, Package, PackageId};

const REQUIRED_LINKS: [&str; 3] = [
    "Official Hardware Repository",
    "Official MicroPython Repository",
    "Official Product Site",
];

const UNIQUE_LINKS: [&str; 4] = [
    "Official Hardware Repository",
    "Official MicroPython Repository",
    "Official Product Site",
    "Datasheet",
];

fn find_definition<'a>(
    node: &'a markdown::mdast::Node,
    needle_label: &str,
) -> Option<&'a Definition> {
    node.children()
        .unwrap()
        .iter()
        .find_map(|node| match *node {
            markdown::mdast::Node::Definition(ref definition)
                if definition.label.as_deref() == Some(needle_label) =>
            {
                Some(definition)
            }
            _ => None,
        })
}

fn find_main_heading(node: &markdown::mdast::Node) -> Option<&Heading> {
    node.children()
        .unwrap()
        .iter()
        .find_map(|node| match *node {
            markdown::mdast::Node::Heading(ref heading) => Some(heading),
            _ => None,
        })
}

fn find_package(packages: &[Package], id: PackageId) -> Option<&Package> {
    packages.iter().find(|package| package.id == id)
}

fn main() {
    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();

    // exclude xtask
    let workspace_members = metadata
        .workspace_members
        .into_iter()
        .filter(|package_id| package_id.repr.starts_with('p'));

    let mut links: HashMap<String, Vec<camino::Utf8PathBuf>> = HashMap::new();
    for member_package_id in workspace_members {
        let package = find_package(&metadata.packages, member_package_id).unwrap();
        let readme_path = package.readme().unwrap();
        let mut file = File::open(readme_path.clone()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mdast = markdown::to_mdast(&contents, &markdown::ParseOptions::gfm()).unwrap();

        for required_link in REQUIRED_LINKS {
            if find_definition(&mdast, required_link).is_none() {
                panic!("{} definition not found in {}", required_link, readme_path);
            }
        }
        for unique_link in UNIQUE_LINKS {
            if let Some(definition) = find_definition(&mdast, unique_link) {
                links
                    .entry(definition.url.clone())
                    .and_modify(|v| v.push(readme_path.clone()))
                    .or_insert(vec![readme_path.clone()]);
            }
        }
        for (url, readme_paths) in links.iter() {
            if readme_paths.len() > 1 {
                panic!("duplicate url {} in {:?}", url, readme_paths);
            }
        }
        if find_main_heading(&mdast).is_none() {
            panic!("no heading in {:?}", readme_path);
        }
    }
}
