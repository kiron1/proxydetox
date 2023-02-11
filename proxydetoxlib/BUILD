load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "proxydetoxlib",
    srcs = [
        "src/accesslog.rs",
        "src/auth.rs",
        "src/auth/kerberos.rs",
        "src/auth/netrc.rs",
        "src/client.rs",
        "src/client/direct.rs",
        "src/client/proxy.rs",
        "src/connect.rs",
        "src/lib.rs",
        "src/net.rs",
        "src/session.rs",
        "src/session/builder.rs",
        "src/session/peer.rs",
        "src/session/shared.rs",
        "src/socket.rs",
    ],
    aliases = aliases(),
    compile_data = [
        "src/502.html",
        "src/accesslog.html",
    ],
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
        "//paclib",
        "//proxy_client",
    ] + all_crate_deps(normal = True),
)
