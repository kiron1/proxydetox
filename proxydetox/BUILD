load("@rules_pkg//:pkg.bzl", "pkg_zip")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

rust_library(
    name = "proxydetoxlib",
    srcs = [
        "src/auth.rs",
        "src/auth/kerberos.rs",
        "src/auth/netrc.rs",
        "src/client.rs",
        "src/io.rs",
        "src/lib.rs",
        "src/net.rs",
        "src/session.rs",
    ],
    compile_data = [
        "src/502.html",
    ],
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    crate_name = "proxydetox",
    proc_macro_deps = [
        "//proxydetox/cargo:tracing_attributes",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//proxydetox/cargo:base64",
        "//proxydetox/cargo:dirs",
        "//proxydetox/cargo:env_logger",
        "//proxydetox/cargo:futures",
        "//proxydetox/cargo:futures_util",
        "//proxydetox/cargo:http",
        "//proxydetox/cargo:hyper",
        "//proxydetox/cargo:netrc",
        "//proxydetox/cargo:parking_lot",
        "//proxydetox/cargo:thiserror",
        "//proxydetox/cargo:tokio",
        "//proxydetox/cargo:tracing",
        "//proxydetox/cargo:tracing_futures",
        "//proxydetox/cargo:tracing_subscriber",
        "//paclib",
        "//proxy_client",
    ] + select({
        "//:enable_negotiate": ["//proxydetox/cargo:cross_krb5"],
        "//conditions:default": [],
    }),
)

rust_binary(
    name = "proxydetox",
    srcs = [
        "src/limit.rs",
        "src/main.rs",
        "src/options.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":proxydetoxlib",
        "//proxydetox/cargo:clap",
        "//proxydetox/cargo:dirs",
        "//proxydetox/cargo:futures",
        "//proxydetox/cargo:futures_util",
        "//proxydetox/cargo:inotify",
        "//proxydetox/cargo:libc",
        "//proxydetox/cargo:tokio",
        "//proxydetox/cargo:tracing",
        "//proxydetox/cargo:tracing_subscriber",
    ],
)

pkg_zip(
    name = "proxydetox.zip",
    srcs = [":proxydetox"],
    tags = ["manual"],
)
