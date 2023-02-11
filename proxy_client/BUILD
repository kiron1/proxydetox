load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "proxy_client",
    srcs = [
        "src/http_connect_connector.rs",
        "src/http_connect_stream.rs",
        "src/http_proxy_connector.rs",
        "src/http_proxy_stream.rs",
        "src/lib.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
    ] + all_crate_deps(
        normal = True,
    ),
)
