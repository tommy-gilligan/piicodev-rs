use core::time::Duration;
use glob::glob;
use http::Uri;
use markdown::mdast::{Definition, Heading, Text};
use reqwest::blocking::ClientBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
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

const TITLE_PREFIX: &str = "Unofficial Rust Driver for PiicoDev ";

fn find_definition<'a>(
    node: &'a markdown::mdast::Node,
    needle_label: &str,
) -> Option<&'a Definition> {
    node.children().unwrap().iter().find_map(|n| match *n {
        markdown::mdast::Node::Definition(ref definition)
            if definition.label.as_deref() == Some(needle_label) =>
        {
            Some(definition)
        }
        _ => None,
    })
}

fn find_main_heading(node: &markdown::mdast::Node) -> Option<&Heading> {
    node.children().unwrap().iter().find_map(|n| match *n {
        markdown::mdast::Node::Heading(ref heading) => Some(heading),
        _ => None,
    })
}

fn find_package<'a>(packages: &'a [Package], id: &'a PackageId) -> Option<&'a Package> {
    packages.iter().find(|package| &(package.id) == id)
}

fn text(node: &Heading) -> Option<String> {
    match node.children.clone().pop() {
        Some(markdown::mdast::Node::Text(Text { value, .. })) => Some(value),
        _ => None,
    }
}

#[allow(clippy::mutable_key_type)]
fn uris() -> HashSet<Uri> {
    #[allow(clippy::mutable_key_type)]
    let mut res: HashSet<Uri> = HashSet::new();
    let excluded_hosts: HashSet<&str> = HashSet::from(["docs.rs", "doc.rust-lang.org"]);
    for file in glob("target/doc/**/*.html")
        .expect("Failed to read glob pattern")
        .filter(core::result::Result::is_ok)
    {
        let binding = std::fs::read_to_string(file.unwrap()).unwrap();
        let dom = tl::parse(&binding, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let a_elements = dom.query_selector("a[href]").unwrap();

        for url in a_elements
            .map(|a_element| {
                a_element
                    .get(parser)
                    .unwrap()
                    .as_tag()
                    .unwrap()
                    .attributes()
                    .get("href")
                    .unwrap()
                    .unwrap()
                    .as_utf8_str()
                    .into_owned()
            })
            .filter_map(|u| u.parse::<Uri>().ok())
            .filter(|u| u.scheme().is_some())
            .filter(|u| !excluded_hosts.contains(u.host().unwrap()))
        {
            res.insert(url);
        }
    }
    res
}

fn check_links() {
    let seconds = Duration::new(10, 0);
    let client = ClientBuilder::new()
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .timeout(seconds)
        .connect_timeout(seconds)
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.4 Safari/605.1.15")
        .build().unwrap();
    for uri in uris() {
        client.head(uri.to_string()).send().map_or_else(
            |_| println!("{uri:?}"),
            |res| {
                if !res.status().is_success() {
                    println!("{:?} {:?}", uri, res.status());
                }
            },
        );
    }
}

fn ci() {
    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();

    // exclude xtask
    let mut workspace_members = metadata
        .workspace_members
        .into_iter()
        .filter(|package_id| package_id.repr.starts_with('p'))
        .collect::<Vec<PackageId>>();
    workspace_members.sort_by_key(|k| {
        return (*(k.repr))
            .strip_prefix('p')
            .unwrap()
            .split(' ')
            .next()
            .unwrap()
            .parse::<u8>()
            .unwrap();
    });

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
        let title: String = text(find_main_heading(&mdast).unwrap()).unwrap();
        let title_without_prefix: &str = title.strip_prefix(TITLE_PREFIX).unwrap();
        let mut components = package.manifest_path.components().rev();
        components.next();
        println!(
            "- [{}](./{}/)",
            title_without_prefix,
            components.next().unwrap()
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

fn main() {
    let cmd = clap::Command::new("xtask")
        .bin_name("xtask")
        .subcommand_required(true)
        .subcommand(clap::command!("ci"))
        .subcommand(clap::command!("check-links"));
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("check-links", _)) => check_links(),
        Some(("ci", _)) => ci(),
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
