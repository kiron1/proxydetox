load("@bazel_skylib//rules:common_settings.bzl", "string_flag", "string_list_flag")

exports_files(
    [".cargo/config.toml"],
    visibility = ["//visibility:public"],
)

string_flag(
     name = "version",
     build_setting_default = "0",
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
