use markdown::mdast::Definition;
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
    needle_label: &'a str,
) -> Option<&'a markdown::mdast::Node> {
    node.children().unwrap().iter().find(|node| match *node {
        markdown::mdast::Node::Definition(Definition {
            label: Some(label), ..
        }) => label == needle_label,
        _ => false,
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
    }
}
