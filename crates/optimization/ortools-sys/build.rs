//! Build script for ortools-sys
//!
//! When the `link` feature is enabled, this will:
//! 1. Compile the C++ wrapper
//! 2. Link against OR-Tools library
//!
//! OR-Tools must be built or installed separately.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.cc");

    #[cfg(feature = "link")]
    build_with_ortools();
}

#[cfg(feature = "link")]
fn build_with_ortools() {
    use std::env;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ortools_root = manifest_dir.parent().unwrap();

    // Check for OR-Tools build directory
    let ortools_build = ortools_root.join("build");
    let ortools_include = ortools_root.to_path_buf();

    if !ortools_build.exists() {
        panic!(
            "OR-Tools build directory not found at {:?}. \
             Please build OR-Tools first with: \
             cmake -S . -B build && cmake --build build",
            ortools_build
        );
    }

    // Compile wrapper
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("wrapper.cc")
        // Source headers
        .include(&ortools_include)
        // Generated headers (protobufs)
        .include(&ortools_build)
        // Abseil headers
        .include(ortools_build.join("_deps/absl-src"))
        // Protobuf headers
        .include(ortools_build.join("_deps/protobuf-src/src"))
        // Protobuf third_party (utf8_range)
        .include(ortools_build.join("_deps/protobuf-src/third_party/utf8_range"))
        // OR_PROTO_DLL is empty on non-Windows (used for DLL export/import)
        .define("OR_PROTO_DLL", "")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-deprecated-declarations")
        .compile("ortools_wrapper");

    // Link directories
    let lib_dir = ortools_build.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Expose lib path to downstream crates via DEP_ORTOOLS_LIB_DIR
    println!("cargo:LIB_DIR={}", lib_dir.display());

    // Link OR-Tools libraries (dynamic)
    println!("cargo:rustc-link-lib=dylib=ortools");

    // Link required abseil libraries (only ones that exist)
    let absl_libs = [
        "absl_log_internal_message",
        "absl_log_internal_check_op",
        "absl_log_internal_nullguard",
        "absl_log_internal_conditions",
        "absl_log_internal_format",
        "absl_log_internal_globals",
        "absl_log_internal_log_sink_set",
        "absl_log_globals",
        "absl_log_entry",
        "absl_log_severity",
        "absl_raw_logging_internal",
        "absl_examine_stack",
        "absl_stacktrace",
        "absl_symbolize",
        "absl_debugging_internal",
        "absl_demangle_internal",
        "absl_demangle_rust",
        "absl_decode_rust_punycode",
        "absl_time",
        "absl_time_zone",
        "absl_civil_time",
        "absl_strings",
        "absl_strings_internal",
        "absl_string_view",
        "absl_int128",
        "absl_throw_delegate",
        "absl_base",
        "absl_spinlock_wait",
        "absl_synchronization",
        "absl_malloc_internal",
        "absl_graphcycles_internal",
        "absl_kernel_timeout_internal",
        "absl_hashtablez_sampler",
        "absl_exponential_biased",
        "absl_hash",
        "absl_city",
        "absl_raw_hash_set",
        "absl_status",
        "absl_statusor",
        "absl_cord",
        "absl_cord_internal",
        "absl_cordz_functions",
        "absl_cordz_handle",
        "absl_cordz_info",
        "absl_cordz_sample_token",
        "absl_crc32c",
        "absl_crc_cord_state",
        "absl_crc_cpu_detect",
        "absl_crc_internal",
        "absl_strerror",
        "absl_random_internal_seed_material",
        "absl_random_seed_gen_exception",
        "absl_random_seed_sequences",
        "absl_random_distributions",
        "absl_random_internal_randen",
        "absl_random_internal_randen_hwaes",
        "absl_random_internal_randen_hwaes_impl",
        "absl_random_internal_randen_slow",
        "absl_random_internal_platform",
        "absl_random_internal_entropy_pool",
        "absl_str_format_internal",
        "absl_tracing_internal",
        "absl_vlog_config_internal",
    ];

    for lib in absl_libs {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    // Link protobuf
    println!("cargo:rustc-link-lib=dylib=protobuf");

    // Link C++ standard library
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");
}
