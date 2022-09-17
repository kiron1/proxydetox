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
        "//detox_net",
        "//duktape",
        "//duktape-sys",
        "//paclib/cargo:gethostname",
        "//paclib/cargo:http",
        "//paclib/cargo:lazy_static",
        "//paclib/cargo:log",
        "//paclib/cargo:regex",
        "//paclib/cargo:thiserror",
    ],
)
