load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_pkg//pkg:providers.bzl", "PackageVariablesInfo")

def _pkg_variables_impl(ctx):
    values = {
        "architecture": ctx.attr.architecture,
        "os": ctx.attr.os,
        "version": ctx.attr.version[BuildSettingInfo].value,
    }
    return PackageVariablesInfo(values = values)

pkg_variables = rule(
    implementation = _pkg_variables_impl,
    attrs = {
        "architecture": attr.string(
            doc = "Architecture of this build.",
        ),
        "os": attr.string(
            doc = "Operating system of this build.",
        ),
        "version": attr.label(
            doc = "Version of this build.",
        ),
    },
    doc = "Collect variables used during package generation.",
)
