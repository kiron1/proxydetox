load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "proxy_client",
    srcs = [
        "src/http_connect_connector.rs",
        "src/http_connect_stream.rs",
        "src/http_proxy_connector.rs",
        "src/http_proxy_stream.rs",
        "src/lib.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
        "//proxy_client/cargo:http",
        "//proxy_client/cargo:hyper",
        "//proxy_client/cargo:thiserror",
        "//proxy_client/cargo:tokio",
        "//proxy_client/cargo:tower",
    ],
)
