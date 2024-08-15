use anyhow::Result;
use clap::Parser;
use functions::{create_build_rs, create_sys_crate, find_source_files};
use std::path::PathBuf;

#[macro_use]
mod macros;
mod functions;

/// cube is a helper for generating Rust bindings for ST Cube HAL C code. Use cube --help to learn more.
#[derive(Parser)]
struct Args {
    /// name of -sys crate, you don't need to add -sys in the argument, simply the name of your crate. For example inputting cube, outputs a cube-sys crate.
    name: String,
    /// Path to the Makefile, this is used to read the C_SOURCES section of the Makefile, so we can select what source files must be used
    cube_project: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if std::fs::read_dir(PathBuf::from(&args.name)).is_err() {
        create_sys_crate(&args.name)?;
    } else {
        println!("{}-sys crate exists, ignoring cargo new...", &args.name);
    }

    println!("Parsing C_SOURCES from Makefile...");
    let sources = find_source_files(&args.name, &args.cube_project)?;

    create_build_rs(&args.name, sources)?;

    Ok(())
}
