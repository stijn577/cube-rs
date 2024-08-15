use anyhow::{Context, Result};
use clap::Parser;
use std::{
    fs::FileType,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Args {
    /// name of -sys crate, you don't need to add -sys in the argument, simply the name of your crate. For example inputting cube, outputs a cube-sys crate.
    name: String,
    /// Path to the Makefile, this is used to read the C_SOURCES section of the Makefile, so we can select what source files must be used
    makefile: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let contents = std::fs::read_to_string(args.makefile)?;

    let sources = find_source_files(contents)?;

    Ok(())
}

fn find_source_files(file_content: String) -> Result<Vec<String>> {
    let vec = file_content
        .lines()
        .map(|line| line.to_owned())
        .enumerate()
        .collect::<Vec<_>>();

    let start;
    let end;

    let mut i = 0;
    while !vec[i].1.starts_with("C_SOURCES") {
        i += 1;
    }
    start = i;

    while vec[i].1.ends_with('\\') {
        i += 1
    }
    end = i;

    Ok(vec
        .into_iter()
        .filter(|(i, _)| start < *i && *i < end + 1)
        .map(|(_, line)| line.trim_matches('\\').trim().to_owned())
        .collect())
}
