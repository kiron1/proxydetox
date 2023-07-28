load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "proxydetoxlib",
    srcs = [
        "src/accesslog.rs",
        "src/auth.rs",
        "src/auth/negotiate.rs",
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
        "//spnego",
    ] + all_crate_deps(normal = True),
)

rust_test(
    name = "proxydetoxlib_test",
    crate = ":proxydetoxlib",
)

env_src = glob(["tests/environment/**/*.rs"])

rust_test(
    name = "proxydetoxlib_connect_direct_test",
    srcs = ["tests/connect_direct.rs"] + env_src,
    crate_root = "tests/connect_direct.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_connect_proxy_test",
    srcs = ["tests/connect_proxy.rs"] + env_src,
    crate_root = "tests/connect_proxy.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_http_api_test",
    srcs = ["tests/http_api.rs"] + env_src,
    crate_root = "tests/http_api.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)
rust_test(
    name = "proxydetoxlib_http_direct_test",
    srcs = ["tests/http_direct.rs"] + env_src,
    crate_root = "tests/http_direct.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_http-proxy_test",
    srcs = ["tests/http_proxy.rs"] + env_src,
    crate_root = "tests/http_proxy.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_net_test",
    srcs = ["tests/net.rs"] + env_src,
    crate_root = "tests/net.rs",
    deps = [
        ":proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)
