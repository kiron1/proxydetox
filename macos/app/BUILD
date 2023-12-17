load("@build_bazel_rules_apple//apple:macos.bzl", "macos_application")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")
load("@rules_pkg//:pkg.bzl", "pkg_zip")

# https://github.com/bazelbuild/rules_swift/blob/master/doc/rules.md#swift_library
swift_library(
    name = "Proxydetox",
    srcs = [
        "AppDelegate.swift",
        "InternalNetworkController.swift",
        "ProxydetoxControl.swift",
        "StatusItemController.swift",
        "main.swift",
    ],
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:private"],
)

# https://github.com/bazelbuild/rules_apple/blob/master/doc/rules-macos.md#macos_application
macos_application(
    name = "ProxydetoxApp",
    additional_contents = {
        "//macos/launcher:ProxydetoxLauncher": "Library/LoginItems",
        "//proxydetox:proxydetoxcli": "MacOS",
    },
    bundle_id = "cc.colorto.Proxydetox",
    entitlements = ":ProxydetoxApp.entitlements",
    infoplists = ["Info.plist"],
    minimum_os_version = "11.1",
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
    deps = [
        ":Proxydetox",
    ],
)

pkg_zip(
    name = "proxydetoxapp.zip",
    srcs = [":ProxydetoxApp"],
    tags = ["manual"],
)
