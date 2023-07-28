load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "proxy_client",
    srcs = [
        "src/http_connect_connector.rs",
        "src/http_connect_stream.rs",
        "src/http_proxy_connector.rs",
        "src/http_proxy_stream.rs",
        "src/lib.rs",
        "src/stream.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
        "//paclib",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "proxy_client_test",
    crate = ":proxy_client",
)
