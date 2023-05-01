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

fn find_package<'a>(packages: &'a [Package], id: &'a PackageId) -> Option<&'a Package> {
    packages.iter().find(|package| &(package.id) == id)
}

fn main() {
    let cmd = clap::Command::new("xtask")
        .bin_name("xtask")
        .subcommand_required(true)
        .subcommand(clap::command!("ci"));
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("ci", _)) => {
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
            let mut all_keywords: HashMap<String, u8> = HashMap::new();
            for member_package_id in workspace_members {
                let package = find_package(&metadata.packages, &member_package_id).unwrap();

                let keywords = &mut package.keywords.clone();
                keywords.sort();
                all_keywords
                    .entry(keywords.join(","))
                    .and_modify(|v| *v += 1)
                    .or_insert_with(|| 1);

                let readme_path = package.readme().unwrap();
                let mut file = File::open(readme_path.clone()).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let mdast = markdown::to_mdast(&contents, &markdown::ParseOptions::gfm()).unwrap();

                for required_link in REQUIRED_LINKS {
                    assert!(
                        find_definition(&mdast, required_link).is_some(),
                        "{required_link} definition not found in {readme_path}"
                    );
                }
                for unique_link in UNIQUE_LINKS {
                    if let Some(definition) = find_definition(&mdast, unique_link) {
                        links
                            .entry(definition.url.clone())
                            .and_modify(|v| v.push(readme_path.clone()))
                            .or_insert_with(|| vec![readme_path.clone()]);
                    }
                }
                assert!(
                    find_main_heading(&mdast).is_some(),
                    "no heading in {readme_path:?}"
                );
            }
            for (url, readme_paths) in &links {
                assert!(
                    readme_paths.len() <= 1,
                    "duplicate url {url} in {readme_paths:?}"
                );
            }
            for (keywords, len) in &all_keywords {
                assert!(*len <= 1, "duplicate keywords {keywords}");
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
