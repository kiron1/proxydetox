load("@bazel_skylib//rules:common_settings.bzl", "string_list_flag")

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
