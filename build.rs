extern crate bindgen;
extern crate pkg_config;

use std::{env, fs, path, process};

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
            let vec = env::var("VEC").unwrap_or("avx2".to_string());

            let dst = path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("astc-encoder/Source");

            process::Command::new("make")
                .env("CXXFLAGS", "-static")
                .current_dir(&dst)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            let name = format!("astcenc-{}", vec);

            let mut out = out_path.join(&name);
            out.set_extension("a");
            let mut file_name = String::from("lib");
            file_name.push_str(out.file_name().unwrap().to_str().unwrap());
            out.set_file_name(file_name);

            fs::copy(dst.join(&name), &out).unwrap();

            println!("cargo:rustc-link-path={}", out.display());

            vec![dst.display().to_string()]
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

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-changed=build.rs");
}
