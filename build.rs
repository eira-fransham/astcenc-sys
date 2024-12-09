use std::{env, path};

fn main() {
    let out_path = path::PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let include_paths = match pkg_config::Config::new().probe("astc-encoder") {
        Ok(astcenc) => {
            for path in astcenc.link_paths {
                println!("cargo:rustc-link-path={}", path.to_str().unwrap());
            }
            for lib in astcenc.libs {
                println!("cargo:rustc-link-lib={}", lib);
            }

            astcenc
                .include_paths
                .into_iter()
                .map(|p| p.into_os_string().into_string().unwrap())
                .collect::<Vec<_>>()
        }
        _ => {
            let source_root = path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("astc-encoder");

            // See <https://github.com/ARM-software/astc-encoder/blob/main/CMakeLists.txt>.
            let dst_root = cmake::Config::new(&source_root)
                .define("ASTCENC_UNIVERSAL_BUILD", "OFF")
                .define("ASTCENC_ISA_NATIVE", "ON")
                .build();

            println!("cargo:rustc-link-lib=astcenc-native-static");
            // Non-Windows.
            println!(
                "cargo:rustc-link-search={}",
                dst_root.join("build").join("Source").display()
            );
            // Windows.
            println!(
                "cargo:rustc-link-search={}",
                dst_root
                    .join("build")
                    .join("Source")
                    .join("Release")
                    .display()
            );

            vec![source_root.join("Source").display().to_string()]
        }
    };

    // Link to libstdc++ on GNU
    let target = env::var("TARGET").unwrap();
    if target.contains("gnu") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    }

    let mut bindings = bindgen::Builder::default()
        .clang_arg("-xc++")
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)
        .formatter(bindgen::Formatter::Prettyplease)
        // Bypasses an issue with bindgen that makes it generate invalid Rust code.
        .blocklist_item("std::value");

    for path in include_paths {
        bindings = bindings.clang_args(&["-F", &path]);
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-changed=build.rs");
}
