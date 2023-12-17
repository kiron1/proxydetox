load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_pkg//pkg:providers.bzl", "PackageVariablesInfo")

def _pkg_variables_impl(ctx):
    values = {
        "architecture": ctx.attr.architecture,
        "os": ctx.attr.os,
        "rev": ctx.attr.rev[BuildSettingInfo].value,
        "version": ctx.attr.version[BuildSettingInfo].value,
    }
    return [
        PackageVariablesInfo(values = values),
        platform_common.TemplateVariableInfo({k.upper(): v for k, v in values.items()}),
    ]

pkg_variables = rule(
    implementation = _pkg_variables_impl,
    attrs = {
        "architecture": attr.string(
            doc = "Architecture of this build.",
        ),
        "os": attr.string(
            doc = "Operating system of this build.",
        ),
        "rev": attr.label(
            doc = "Revision of this build.",
        ),
        "version": attr.label(
            doc = "Version of this build.",
        ),
    },
    doc = "Collect variables used during package generation.",
)
