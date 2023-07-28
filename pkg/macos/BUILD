load("@//bazel:pkgbuild.bzl", "pkgbuild")
load("@build_bazel_rules_apple//apple:macos.bzl", "macos_command_line_application")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")

swift_library(
    name = "setproxylib",
    srcs = [
        "setproxy.swift",
    ],
    target_compatible_with = ["@platforms//os:macos"],
)

macos_command_line_application(
    name = "setproxy",
    bundle_id = "cc.colorto.SetProxy",
    minimum_os_version = "11.1",
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
    deps = [
        ":setproxylib",
    ],
)

sh_binary(
    name = "proxydetoxctl_bin",
    srcs = ["proxydetoxctl"],
)

pkg_tar(
    name = "bin",
    srcs = [
        ":proxydetoxctl_bin",
        "//proxydetox",
    ],
    package_dir = "opt/proxydetox/bin",
)

pkg_tar(
    name = "libexec",
    srcs = [
        ":setproxy",
    ],
    package_dir = "opt/proxydetox/libexec",
)

pkg_tar(
    name = "pathsd",
    srcs = [":40-proxydetox"],
    package_dir = "etc/paths.d",
)

pkg_tar(
    name = "launch_argents",
    srcs = [":cc.colorto.proxydetox.plist"],
    package_dir = "Library/LaunchAgents",
)

pkg_tar(
    name = "data",
    deps = [
        ":bin",
        ":launch_argents",
        ":libexec",
        ":pathsd",
    ],
)

pkgbuild(
    name = "macos",
    out = "proxydetox.pkg",
    data = ":data",
    identifier = "cc.colorto.proxydetox",
    install_location = "/",
    package_file_name = "proxydetox_{version}_{architecture}.deb",
    package_variables = "//pkg:variables",
    postinstall = ":postinstall",
    preinstall = ":preinstall",
    version_file = "//pkg:version",
    visibility = ["//visibility:public"],
)
