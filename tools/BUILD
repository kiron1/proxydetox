load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "toml_get",
    srcs = [
        "toml_get.rs",
    ],
    visibility = ["//visibility:public"],
    deps = ["@crate_index_tools//:toml"],
)
