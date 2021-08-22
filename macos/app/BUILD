load("@build_bazel_rules_apple//apple:macos.bzl", "macos_application")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

# https://github.com/bazelbuild/rules_swift/blob/master/doc/rules.md#swift_library
swift_library(
    name = "Proxydetox",
    srcs = [
        "AppDelegate.swift",
        "ProxydetoxControl.swift",
        "StatusItemCOntroller.swift",
        "main.swift",
    ],
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:private"],
    deps = ["//cproxydetox:swiftlyproxydetox"],
)

# https://github.com/bazelbuild/rules_apple/blob/master/doc/rules-macos.md#macos_application
macos_application(
    name = "ProxydetoxApp",
    additional_contents = {
        "//macos/launcher:ProxydetoxLauncher": "Library/LoginItems",
    },
    bundle_id = "com.github.kiron1.Proxydetox",
    entitlements = ":ProxydetoxApp.entitlements",
    infoplists = ["Info.plist"],
    linkopts = [
        "-framework",
        "GSS",
    ],
    minimum_os_version = "10.14",
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
    deps = [
        ":Proxydetox",
    ],
)
