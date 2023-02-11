load("@rules_rust//bindgen:bindgen.bzl", "rust_bindgen")
load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

cc_library(
    name = "duktapelib",
    srcs = [
        "src/duktape.c",
    ],
    hdrs = [
        "src/duk_config.h",
        "src/duktape.h",
    ],
    includes = ["src"],
    visibility = ["//visibility:private"],
)

rust_bindgen(
    name = "bindings",
    bindgen_flags = [
        "--allowlist-function=duk_.*",
        "--allowlist-type=duk_.*",
        "--allowlist-var=DUK_.*",
    ],
    cc_lib = ":duktapelib",
    clang_flags = [
    ] + select({
        "@bazel_tools//src/conditions:darwin": [
            # TODO: https://github.com/bazelbuild/rules_rust/issues/899
            "-isystem",
            "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include",
        ],
        "//conditions:default": [],
    }),
    header = ":src/duktape.h",
    visibility = ["//visibility:private"],
)

rust_library(
    name = "duktape-sys",
    srcs = [
        "src/lib.rs",
    ],
    aliases = aliases(),
    compile_data = [
        ":bindings",
    ],
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    rustc_env = {
        "DUKTAPE_BINDINGS_RS": "$(execpath :bindings)",
    },
    visibility = ["//visibility:public"],
    deps = [":duktapelib"] + all_crate_deps(
        normal = True,
    ),
)
