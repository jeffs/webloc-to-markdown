// Standard library
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{env, io};

// Third party
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

struct Extractor {
    re: Regex,
}

impl Extractor {
    pub fn new() -> Extractor {
        Extractor {
            re: Regex::new(r"^\t<string>(.*)</string>$").unwrap(),
        }
    }

    pub fn link(&self, line: &str) -> Option<String> {
        self.re
            .captures(&line)
            .and_then(|captures| captures.get(1).map(|m| m.as_str().to_string()))
    }
}

fn is_webloc(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .filter(|&ext| ext == "webloc")
        .is_some()
}

fn find_weblocs(root: String) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(is_webloc)
        .map(|entry| entry.path().to_path_buf())
}

fn read_link(path: &Path) -> Option<String> {
    let extract = Extractor::new();
    match File::open(path) {
        Ok(file) => {
            let links = io::BufReader::new(file)
                .lines()
                .flatten()
                .filter_map(|line| extract.link(&line))
                .collect::<Vec<String>>();
            match links.as_slice() {
                [link] => Some(link.to_string()),
                _ => None,
            }
        }
        Err(_) => None,
    }
}

fn get_stem(path: &Path) -> &str {
    path.file_stem()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("(non_utf8_file_name)")
}

fn main() {
    let paths: Vec<PathBuf> = env::args().skip(1).flat_map(find_weblocs).collect();
    let link_opts = paths.iter().map(|path| read_link(&path));
    for (path, link_opt) in paths.iter().zip(link_opts) {
        match link_opt {
            Some(link) => {
                println!("* [{}]( {} )", get_stem(path), link);
            }
            None => {
                eprintln!("warning: failed to parse {}", path.display());
            }
        }
    }
}
