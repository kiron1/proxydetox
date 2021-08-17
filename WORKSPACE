workspace(name = "proxydetox")

load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# TODO: proper integration of Rust code
# For this to work,
#  env -i PATH=$HOME/.cargo/bin:/bin:/usr/bin http_proxy=$http_proxy https_proxy=$https_proxy cargo build --release --features negotiate
#  codesign -s "Apple Development: mail@example.org (0123456789)" target/release/proxydetox
# must be run once (and after every code change in rust) before bazel build
# is invoked.
new_local_repository(
    name = "proxydetoxcli",
    path = "target/release",
    build_file_content = "exports_files([\"proxydetox\"])",
)
http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "0052d452af7742c8f3a4e0929763388a66403de363775db7e90adecb2ba4944b",
    url = "https://github.com/bazelbuild/rules_apple/releases/download/0.31.3/rules_apple.0.31.3.tar.gz",
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