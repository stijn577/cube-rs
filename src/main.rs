use anyhow::Result;
use clap::Parser;
use functions::{CubeProjectParser, MxProjectDataRequest};
use std::path::PathBuf;

mod error;
mod functions;

/// cube is a helper for generating Rust bindings for ST Cube HAL C code. Use cube --help to learn more.
#[derive(Parser, Debug)]
struct Args {
    /// name of -sys crate, you don't need to add -sys in the argument, simply the name of your crate. For example inputting cube, outputs a cube-sys crate.
    name: String,
    /// Path to the Makefile, this is used to read the C_SOURCES section of the Makefile, so we can select what source files must be used
    cube_project: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let project = CubeProjectParser::new(args.name, args.cube_project.clone())?;
    project.create_sys_crate()?;

    let sources = project.get_entry(MxProjectDataRequest::Sources)?;
    let headers = project.get_entry(MxProjectDataRequest::Headers)?;
    let includes = project.get_entry(MxProjectDataRequest::Includes)?;
    let defines = project.get_entry(MxProjectDataRequest::Defines)?;

    project.create_build_rs(&sources, &headers, &includes, &defines)?;
    project.create_wrapper_h(&headers)?;

    Ok(())
}
