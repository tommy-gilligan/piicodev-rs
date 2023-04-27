use markdown::mdast::Definition;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::string::String;

use cargo_metadata::{CargoOpt, MetadataCommand, Package, PackageId};

const REQUIRED_LINKS: [&str; 3] = [
    "Official Hardware Repository",
    "Official Software Repository",
    "Official Product Site",
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

    let mut links: HashSet<String> = HashSet::new();
    for member_package_id in workspace_members {
        let package = find_package(&metadata.packages, member_package_id).unwrap();
        let readme_path = package.readme().unwrap();
        let mut file = File::open(readme_path.clone()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mdast = markdown::to_mdast(&contents, &markdown::ParseOptions::gfm()).unwrap();

        for required_link in REQUIRED_LINKS {
            if let Some(definition) = find_definition(&mdast, required_link) {
                if links.contains(&definition.url) {
                    panic!(
                        "duplicate url {} for link {} in {}",
                        definition.url, required_link, readme_path
                    );
                } else {
                    links.insert(definition.url.clone());
                }
            } else {
                panic!("{} definition not found in {}", required_link, readme_path);
            }
        }
    }
}
