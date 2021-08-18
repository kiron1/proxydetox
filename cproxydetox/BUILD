load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "cproxydetox",
    srcs = [
        "src/lib.rs",
    ],
    rustc_flags = [
        "--crate-type=staticlib",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:dirs",
        "//cargo:libc",
        "//cargo:tokio",
        "//proxydetox:proxydetox",
    ],
)
