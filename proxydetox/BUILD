load("@rules_pkg//:pkg.bzl", "pkg_zip")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

rust_library(
    name = "proxydetoxlib",
    srcs = [
        "src/auth.rs",
        "src/auth/kerberos.rs",
        "src/auth/netrc.rs",
        "src/client.rs",
        "src/detox.rs",
        "src/detox/config.rs",
        "src/detox/service.rs",
        "src/detox/session.rs",
        "src/io.rs",
        "src/lib.rs",
        "src/net.rs",
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
        "//cargo:tracing_attributes",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:base64",
        "//cargo:dirs",
        "//cargo:env_logger",
        "//cargo:futures",
        "//cargo:futures_util",
        "//cargo:http",
        "//cargo:hyper",
        "//cargo:netrc",
        "//cargo:thiserror",
        "//cargo:tokio",
        "//cargo:tracing",
        "//cargo:tracing_futures",
        "//cargo:tracing_subscriber",
        "//paclib",
        "//proxy_client",
    ] + select({
        "//:enable_negotiate": ["//libnegotiate"],
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
        "//cargo:clap",
        "//cargo:dirs",
        "//cargo:futures",
        "//cargo:futures_util",
        "//cargo:inotify",
        "//cargo:libc",
        "//cargo:tokio",
        "//cargo:tracing",
        "//cargo:tracing_subscriber",
    ],
)

pkg_zip(
    name = "proxydetox.zip",
    srcs = [":proxydetox"],
    tags = ["manual"],
)
