load("@build_bazel_rules_apple//apple:macos.bzl", "macos_command_line_application")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_static_library", "rust_test")

exports_files(["Cargo.toml"])

rust_binary(
    name = "proxydetox",
    srcs = [
        "src/main.rs",
        "src/options.rs",
    ],
    aliases = aliases(),
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
        "//proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "proxydetox_test",
    srcs = [
        "src/options.rs",
    ],
    aliases = aliases(),
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    data = ["//paceval:example.pac"],
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    deps = [
        "//proxydetoxlib",
        "//detox_net",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_static_library(
    name = "proxydetoxlib_main",
    srcs = [
        "src/main.rs",
        "src/options.rs",
    ],
    aliases = aliases(),
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    crate_root = "src/main.rs",
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    rustc_flags = [
        "--cfg",
        "static_library",
    ],
    deps = [
        "//proxydetoxlib",
        "//detox_net",
    ] + all_crate_deps(
        normal = True,
    ),
)

macos_command_line_application(
    name = "proxydetoxcli",
    bundle_id = "cc.colorto.ProxydetoxCLI",
    # infoplists = ["Info.plist"],
    linkopts = [
        "-framework",
        "Security",
        "-framework",
        "SystemConfiguration",
    ] + select({
        "//:enable_negotiate": [
            "-framework",
            "GSS",
        ],
        "//conditions:default": [],
    }),
    minimum_os_version = "11.1",
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
    deps = [
        ":proxydetoxlib_main",
    ],
)
