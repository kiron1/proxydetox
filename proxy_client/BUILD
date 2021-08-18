load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "proxy_client",
    srcs = [
        "src/http_proxy_connector.rs",
        "src/http_proxy_stream.rs",
        "src/lib.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:http",
        "//cargo:hyper",
        "//cargo:thiserror",
        "//cargo:tokio",
    ],
)
