load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "proxydetoxlib",
    srcs = [
        "src/accesslog.rs",
        "src/context.rs",
        "src/context/builder.rs",
        "src/lib.rs",
        "src/server.rs",
        "src/session.rs",
        "src/socket.rs",
    ],
    aliases = aliases(),
    compile_data = [
        "src/error.html",
        "src/accesslog.html",
    ],
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    rustc_env = {
        "CARGO_PKG_VERSION": "$(VERSION)",
        "PROXYDETOX_BUILD_GIT_HASH": "$(REV)",
    },
    toolchains = ["//pkg:variables"],
    visibility = ["//visibility:public"],
    deps = [
        "//detox_auth",
        "//detox_futures",
        "//detox_hyper",
        "//detox_net",
        "//paclib",
        "//spnego",
    ] + all_crate_deps(normal = True),
)

rust_test(
    name = "proxydetoxlib_test",
    size = "small",
    crate = ":proxydetoxlib",
)

env_src = glob(["tests/environment/**/*.rs"])

rust_test(
    name = "proxydetoxlib_connect_direct_test",
    size = "small",
    srcs = ["tests/connect_direct.rs"] + env_src,
    crate_root = "tests/connect_direct.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_connect_proxy_test",
    size = "small",
    srcs = ["tests/connect_proxy.rs"] + env_src,
    crate_root = "tests/connect_proxy.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_http_api_test",
    size = "small",
    srcs = ["tests/http_api.rs"] + env_src,
    crate_root = "tests/http_api.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_http_direct_test",
    size = "small",
    srcs = ["tests/http_direct.rs"] + env_src,
    crate_root = "tests/http_direct.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_http-proxy_test",
    size = "small",
    srcs = ["tests/http_proxy.rs"] + env_src,
    crate_root = "tests/http_proxy.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

rust_test(
    name = "proxydetoxlib_net_test",
    size = "small",
    srcs = ["tests/net.rs"] + env_src,
    crate_root = "tests/net.rs",
    deps = [
        ":proxydetoxlib",
        "//detox_auth",
        "//detox_hyper",
    ] + all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)
