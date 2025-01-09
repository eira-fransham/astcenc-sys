use std::{env, path};

fn main() {
    let out_path = path::PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut build = cc::Build::new();

    build.files([
        "astc-encoder/Source/astcenc_averages_and_directions.cpp",
        "astc-encoder/Source/astcenc_block_sizes.cpp",
        "astc-encoder/Source/astcenc_color_quantize.cpp",
        "astc-encoder/Source/astcenc_color_unquantize.cpp",
        "astc-encoder/Source/astcenc_compress_symbolic.cpp",
        "astc-encoder/Source/astcenc_compute_variance.cpp",
        "astc-encoder/Source/astcenc_decompress_symbolic.cpp",
        "astc-encoder/Source/astcenc_diagnostic_trace.cpp",
        "astc-encoder/Source/astcenc_entry.cpp",
        "astc-encoder/Source/astcenc_find_best_partitioning.cpp",
        "astc-encoder/Source/astcenc_ideal_endpoints_and_weights.cpp",
        "astc-encoder/Source/astcenc_image.cpp",
        "astc-encoder/Source/astcenc_integer_sequence.cpp",
        "astc-encoder/Source/astcenc_mathlib.cpp",
        "astc-encoder/Source/astcenc_mathlib_softfloat.cpp",
        "astc-encoder/Source/astcenc_partition_tables.cpp",
        "astc-encoder/Source/astcenc_percentile_tables.cpp",
        "astc-encoder/Source/astcenc_pick_best_endpoint_format.cpp",
        "astc-encoder/Source/astcenc_quantization.cpp",
        "astc-encoder/Source/astcenc_symbolic_physical.cpp",
        "astc-encoder/Source/astcenc_weight_align.cpp",
        "astc-encoder/Source/astcenc_weight_quant_xfer_tables.cpp",
    ]);

    build.compile("astcenc-vendored");

    let main_header = "wrapper.h";

    let bindings = bindgen::Builder::default()
        .clang_arg("-xc++")
        .clang_arg("-Iastc-encoder/Source")
        .header(main_header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)
        .formatter(bindgen::Formatter::Prettyplease)
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings");

    // Link to libstdc++ on GNU
    let target = env::var("TARGET").unwrap();
    if target.contains("gnu") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
