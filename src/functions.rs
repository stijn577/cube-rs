use itertools::Itertools;

use crate::error::CubeParseError::{self, *};
use std::{fs::File, io::Write, path::PathBuf, process::Command};

type CubeResult<T> = Result<T, CubeParseError>;

pub enum MxProjectDataRequest {
    Sources,
    Headers,
    Includes,
    Defines,
}

#[derive(Debug)]
pub struct CubeProjectParser {
    libname: String,
    cube_proj: PathBuf,
    cached_mxproject: Vec<(usize, String)>,
}

impl CubeProjectParser {
    pub fn new(libname: String, project: PathBuf) -> CubeResult<CubeProjectParser> {
        let mut mxproject = project.clone();
        mxproject.push(".mxproject");
        let mxproject = mxproject.canonicalize().unwrap();

        let cached = std::fs::read_to_string(mxproject.clone())
            .map_err(|_| FileNotFound(mxproject))?
            .lines()
            .map(|line| line.to_owned())
            .enumerate()
            .collect::<Vec<_>>();

        Ok(Self {
            libname,
            cube_proj: project,
            cached_mxproject: cached,
        })
    }

    pub fn create_sys_crate(&self) -> CubeResult<()> {
        let mut crate_name = self.libname.clone();
        crate_name.push_str("-sys");
        println!("Checking if crate {} already exists...", crate_name);
        if std::fs::read_dir(&crate_name).is_err() {
            println!("{} does not yet exist, creating it now...", crate_name);
            let mut commands = vec![];
            commands.push(
                Command::new("cargo")
                    .args(["new", "--lib", &crate_name])
                    .output(),
            );
            commands.push(
                Command::new("cargo")
                    .args(["add", "cty"])
                    .current_dir(&crate_name)
                    .output(),
            );
            commands.push(
                Command::new("cargo")
                    .args(["add", "--build", "bindgen", "cc"])
                    .current_dir(&crate_name)
                    .output(),
            );

            for command in commands {
                let dbg = format!("{:?}", command);
                command.map_err(|_| CargoFailed(dbg))?;
            }
        } else {
            println!("{} exists already, ignoring...", crate_name)
        }

        Ok(())
    }

    pub fn get_entry(&self, entry_type: MxProjectDataRequest) -> CubeResult<Vec<String>> {
        let (section, entry) = match entry_type {
            MxProjectDataRequest::Sources => ("[PreviousUsedMakefileFiles]", "SourceFiles="),
            MxProjectDataRequest::Headers => ("[PreviousLibFiles]", "LibFiles="),
            MxProjectDataRequest::Includes => ("[PreviousUsedMakefileFiles]", "HeaderPath="),
            MxProjectDataRequest::Defines => ("[PreviousUsedMakefileFiles]", "CDefines="),
        };

        let entry_data = self.get_entry_data(section, entry)?;
        let entry_data = self.fix_entry(entry_type, entry_data);
        let entry_data = entry_data.into_iter().unique().collect();

        Ok(entry_data)
    }

    fn get_entry_data(&self, section: &str, entry: &str) -> CubeResult<Vec<String>> {
        Ok(self
            .cached_mxproject
            .iter()
            .skip_while(|(_, line)| line.trim() != section)
            .skip(1)
            .find(|(_, line)| line.trim().starts_with(entry))
            .ok_or(EntryNotFound(section.to_owned(), entry.to_owned()))?
            .1
            .split_once('=')
            .ok_or(EntryParse(entry.to_owned()))?
            .1
            .split(';')
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect())
    }

    fn fix_entry(&self, data: MxProjectDataRequest, vec: Vec<String>) -> Vec<String> {
        match data {
            MxProjectDataRequest::Sources => vec
                .into_iter()
                .filter(|s| s.ends_with(".c"))
                .map(|s| format!("..\\{}\\{}", self.cube_proj.display(), s))
                .collect(),
            MxProjectDataRequest::Headers => vec
                .into_iter()
                .filter(|s| s.ends_with(".h"))
                .map(|s| format!("..\\{}\\{}", self.cube_proj.display(), s))
                .collect(),
            MxProjectDataRequest::Includes => vec
                .into_iter()
                .map(|s| format!("-I..\\{}\\{}", self.cube_proj.display(), s))
                .collect(),
            MxProjectDataRequest::Defines => vec.into_iter().map(|s| format!("-D{}", s)).collect(),
        }
    }

    // pub fn get_entry(&self, entry: Entry) -> CubeResult<Vec<String>> {
    //     let prefix = match entry {
    //         "CDefines=" => Ok(String::from("-D")),
    //         "HeaderPath=" => Ok(format!("-I..\\{}\\", self.cube_proj.display())),
    //         _ => Ok(String::new()),
    //     }?;
    //     Ok(self
    //         .cached_mxproject
    //         .iter()
    //         .find(|(_, line)| line.starts_with(entry))
    //         .ok_or(EntryNotFound(entry.to_owned()))?
    //         .1
    //         .split_once('=')
    //         .ok_or(EntryParse(entry.to_owned()))?
    //         .1
    //         .split(';')
    //         .map(|s| format!("{}{}", prefix, s))
    //         .collect())
    // }
    // pub fn get_list(&self, entry_list: &str) -> Result<Vec<PathBuf>> {
    //     let entry_line = self
    //         .cached
    //         .iter()
    //         .find(|(_, line)| line.contains(entry_list))
    //         .ok_or(EntryNotFound(entry_list.to_owned()))?;
    //     let n = entry_line
    //         .1
    //         .split_once('=')
    //         .ok_or(EntryNotFound(entry_list.to_owned()))?
    //         .1
    //         .parse()
    //         .map_err(|_| EntryParse(entry_list.to_owned()))?;
    //     Ok(self
    //         .cached
    //         .iter()
    //         .skip((entry_line.0) + 1)
    //         .take(n)
    //         .map(|(_, line)| {
    //             PathBuf::from(format!(
    //                 "..\\{}\\{}",
    //                 self.project.display(),
    //                 line.trim()
    //                     .split_once('=')
    //                     .unwrap()
    //                     .1
    //                     .strip_prefix("..\\")
    //                     .unwrap()
    //             ))
    //         })
    //         .collect())
    // }

    pub fn create_build_rs(
        &self,
        sources: &[String],
        _headers: &[String],
        include_paths: &[String],
        c_defines: &[String],
    ) -> CubeResult<()> {
        let build_script = format!(
            r#"/* Generated by cube-rs */
use std::env;
use std::path::PathBuf;

fn main() {{
    let c_sources = {0:#?};
    let include_paths = {1:#?};
    let c_defines = {2:#?};

    // create the cube library
    let mut build = cc::Build::new();
    for entry in c_sources {{
        build.file(entry);
    }}
    for path in include_paths {{
        let path = path.strip_prefix("-I").unwrap();
        build.include(PathBuf::from(path));
    }}
    for define in c_defines {{
        let define = define.strip_prefix("-D").unwrap();
        build.define(define, None);
    }}
    build.compile("{3}");

    println!("cargo:rustc-link-lib=static={3}");
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rerun-if-changed=lib{3}.a");


    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("cty")
        .clang_args(include_paths)
        .clang_args(c_defines)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}}"#,
            sources, include_paths, c_defines, self.libname,
        );

        let mut crate_name = self.libname.clone();
        crate_name.push_str("-sys");
        let mut file = PathBuf::from(crate_name);
        file.push("build.rs");
        let mut file = File::create(file).map_err(|_| BuildRsCreate)?;
        file.write_all(build_script.as_bytes()).unwrap();

        Ok(())
    }

    pub fn create_wrapper_h(&self, headers: &[String]) -> CubeResult<()> {
        let headers = headers
            .iter()
            .map(|path| format!("#include \"{}\"\n", path))
            .collect::<Vec<_>>();

        let mut crate_name = self.libname.clone();
        crate_name.push_str("-sys");
        let mut file = PathBuf::from(crate_name);
        file.push("wrapper.h");
        let mut file = File::create(file).map_err(|_| WrapperHCreate)?;

        for header in headers {
            file.write_all(header.as_bytes()).unwrap();
        }

        Ok(())
    }
}
