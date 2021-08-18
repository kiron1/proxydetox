load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "paclib",
    srcs = [
        "src/dns.rs",
        "src/evaluator.rs",
        "src/lib.rs",
        "src/proxy.rs",
    ],
    compile_data = [
        "src/pac_utils.js",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:http",
        "//cargo:lazy_static",
        "//cargo:log",
        "//cargo:regex",
        "//cargo:thiserror",
        "//duktape",
        "//duktape-sys",
    ],
)
