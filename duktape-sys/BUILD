load("@rules_rust//bindgen:bindgen.bzl", "rust_bindgen")
load("@rules_rust//rust:defs.bzl", "rust_library")

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
    compile_data = [
        ":bindings",
    ],
    rustc_env = {
        "DUKTAPE_BINDINGS_RS": "$(execpath :bindings)",
    },
    visibility = ["//visibility:public"],
    deps = [
        ":duktapelib",
    ],
)
