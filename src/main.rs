use anyhow::Result;
use clap::Parser;
use functions::CubeProjectParser;
use std::path::PathBuf;

#[macro_use]
mod macros;
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
    project.makefile_variable("C_SOURCES")?;
    project.makefile_variable("C_INCLUDES")?;

    Ok(())
}
