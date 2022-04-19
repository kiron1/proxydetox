load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "detox_net",
    srcs = [
        "src/host_and_port.rs",
        "src/lib.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//proxy_client/cargo:http",
        "//proxy_client/cargo:thiserror",
    ],
)
