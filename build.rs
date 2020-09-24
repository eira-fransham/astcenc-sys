extern crate bindgen;
extern crate pkg_config;

use std::{env, process};

fn main() {
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
            let vec = env::var("VEC").unwrap_or("avx2".to_string());

            let dst = "astc-encoder/Source";

            process::Command::new("make")
                .current_dir(dst)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            println!("cargo:rustc-link-search=native={}", dst);
            println!("cargo:rustc-link-lib=dylib=astc-encoder-{}", vec);

            vec!["astc-encoder/Source".to_string()]
        }
    };

    // Link to libstdc++ on GNU
    let target = env::var("TARGET").unwrap();
    if target.contains("gnu") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let mut bindings = bindgen::Builder::default()
        .clang_arg("-xc++")
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true);

    for path in include_paths {
        bindings = bindings.clang_args(&["-F", &path]);
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-changed=build.rs");
}
