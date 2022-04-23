workspace(name = "proxydetox")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "4161b2283f80f33b93579627c3bd846169b2d58848b0ffb29b5d4db35263156a",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_apple/releases/download/0.34.0/rules_apple.0.34.0.tar.gz",
        "https://github.com/bazelbuild/rules_apple/releases/download/0.34.0/rules_apple.0.34.0.tar.gz",
    ],
)

load(
    "@build_bazel_rules_apple//apple:repositories.bzl",
    "apple_rules_dependencies",
)

apple_rules_dependencies()

load(
    "@build_bazel_rules_swift//swift:repositories.bzl",
    "swift_rules_dependencies",
)

swift_rules_dependencies()

load(
    "@build_bazel_rules_swift//swift:extras.bzl",
    "swift_rules_extra_dependencies",
)

swift_rules_extra_dependencies()

load(
    "@build_bazel_apple_support//lib:repositories.bzl",
    "apple_support_dependencies",
)

apple_support_dependencies()

http_archive(
    name = "rules_rust",
    sha256 = "b58c63a6d8221f408f8852b4f74f81bc8c7aac9273f3899a74e32e6168a2c624",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_rust/releases/download/0.2.1/rules_rust-v0.2.1.tar.gz",
        "https://github.com/bazelbuild/rules_rust/releases/download/0.2.1/rules_rust-v0.2.1.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2021",
    version = "1.60.0",
)

load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_repositories")

rust_bindgen_repositories()

load("//cargo:crates.bzl", "raze_fetch_remote_crates")

raze_fetch_remote_crates()

http_archive(
    name = "rules_pkg",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/0.7.0/rules_pkg-0.7.0.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/0.7.0/rules_pkg-0.7.0.tar.gz",
    ],
    sha256 = "8a298e832762eda1830597d64fe7db58178aa84cd5926d76d5b744d6558941c2",
)

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()
