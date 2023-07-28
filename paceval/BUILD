load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

exports_files(["example.pac"])

rust_binary(
    name = "paceval",
    srcs = [
        "src/main.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = ["//paclib"] + all_crate_deps(
        normal = True,
    ),
)
