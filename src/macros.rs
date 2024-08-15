macro_rules! build_rs_gen {
    ($wrapper_h: literal) => {
        fn main() {
            // create the cube library
            let mut build = cc::Build::new();
            for entry in $c_sources {
                build.file(entry.unwrap());
            }
            build.compile("cbinder");

            println!("cargo:rustc-link-lib=static={}");
            println!("cargo:rustc-link-search=native=.");
            println!("cargo:rerun-if-changed=lib{}.a");

            // The bindgen::Builder is the main entry point
            // to bindgen, and lets you build up options for
            // the resulting bindings.
            let bindings = bindgen::Builder::default()
                // The input header we would like to generate
                // bindings for.
                .header($wrapper_h)
                .use_core()
                // .clang_arg("-DDEBUG -DUSE_FULL_LL_DRIVER -DUSE_HAL_DRIVER -DSTM32U575xx -c -I../Core/Inc -I../Drivers/STM32U5xx_HAL_Driver/Inc -I../Drivers/STM32U5xx_HAL_Driver/Inc/Legacy -I../Drivers/CMSIS/Device/ST/STM32U5xx/Include -I../Drivers/CMSIS/Include -Os -ffunction-sections -fdata-sections -Wall -fstack-usage --specs=nano.specs -mfpu=fpv5-sp-d16 -mfloat-abi=hard -mthumb")
                .ctypes_prefix("cty")
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
        }
    };
}
