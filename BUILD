load("@bazel_skylib//rules:common_settings.bzl", "string_flag", "string_list_flag")
load("@bazel_skylib//rules:native_binary.bzl", "native_binary")

exports_files(
    [
        ".cargo/config.toml",
        "Cargo.toml",
        "book.toml",
    ],
    visibility = ["//visibility:public"],
)

string_flag(
    name = "version",
    build_setting_default = "0",
    visibility = ["//visibility:public"],
)

string_flag(
    name = "rev",  # `git rev-parse --short=10 HEAD`
    build_setting_default = "unknown",
    visibility = ["//visibility:public"],
)

string_list_flag(
    name = "features",
    build_setting_default = [],
)

config_setting(
    name = "enable_negotiate",
    flag_values = {
        ":features": "negotiate",
    },
)

platform(
    name = "x86_64-apple-darwin",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:macos",
    ],
)

platform(
    name = "aarch64-apple-darwin",
    constraint_values = [
        "@platforms//cpu:arm64",
        "@platforms//os:macos",
    ],
)

native_binary(
    name = "current_version",
    src = "//tools:toml_get",
    out = "current_version",
    args = [
        "-f$(location //:Cargo.toml)",
        "-nversion",
        "workspace",
        "package",
        "version",
    ],
    data = ["//:Cargo.toml"],
    visibility = ["//visibility:public"],
)
