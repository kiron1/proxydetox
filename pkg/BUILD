load("@//bazel:pkg.bzl", "pkg_variables")

alias(
    name = "pkg",
    actual =
        select({
            "@platforms//os:linux": "//pkg/deb",
            "@platforms//os:macos": "//pkg/macos",
            "@platforms//os:windows": "//pkg/windows",
            # Build a zip file for any other platforms:
            "//conditions:default": "//pkg/zip",
        }),
)

pkg_variables(
    name = "variables",
    architecture = select({
        "@platforms//cpu:x86_64": "amd64",
        "@platforms//cpu:aarch64": "aarch64",
    }),
    os = select({
        "@platforms//os:linux": "linux",
        "@platforms//os:macos": "macos",
        "@platforms//os:windows": "windows",
        "//conditions:default": "unknown",
    }),
    version = "//:version",
    visibility = ["//pkg:__subpackages__"],
)

genrule(
    name = "version",
    srcs = ["//proxydetox:Cargo.toml"],
    outs = ["version.txt"],
    cmd = "./$(location //tools:toml_get) $(location //proxydetox:Cargo.toml) package version > \"$@\"",
    tools = ["//tools:toml_get"],
    visibility = ["//visibility:public"],
)
