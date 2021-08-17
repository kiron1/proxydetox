load("@build_bazel_rules_apple//apple:macos.bzl", "macos_application")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

swift_library(
    name = "launcher",
    srcs = [
         "AppDelegate.swift",
    ],
    visibility = ["//visibility:private"],
)

macos_application(
    name = "ProxydetoxLauncher",
    bundle_id = "com.github.kiron1.ProxydetoxLauncher",
    infoplists = ["Info.plist"],
    minimum_os_version = "10.14",
    deps = [
        ":launcher",
    ],
    visibility = ["//visibility:public"],
)
