workspace(name = "proxydetox")

load("@//bazel:http.bzl", "versioned_http_archive")

versioned_http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "43737f28a578d8d8d7ab7df2fb80225a6b23b9af9655fcdc66ae38eb2abcf2ed",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_apple/releases/download/{version}/rules_apple.{version}.tar.gz",
        "https://github.com/bazelbuild/rules_apple/releases/download/{version}/rules_apple.{version}.tar.gz",
    ],
    version = "2.0.0",
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

versioned_http_archive(
    name = "rules_rust",
    sha256 = "2466e5b2514772e84f9009010797b9cd4b51c1e6445bbd5b5e24848d90e6fb2e",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz",
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz",
    ],
    version = "0.18.0",
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2021",
    # version = "1.67.0",
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_config = "//:.cargo/config.toml",
    cargo_lockfile = "//:Cargo.lock",
    # Generate with:
    # CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index
    lockfile = "//:Cargo.Bazel.lock",
    manifests = [
        "//:Cargo.toml",
        "//cproxydetox:Cargo.toml",
        "//detox_net:Cargo.toml",
        "//dnsdetox:Cargo.toml",
        "//duktape:Cargo.toml",
        "//duktape-sys:Cargo.toml",
        "//paceval:Cargo.toml",
        "//paclib:Cargo.toml",
        "//proxy_client:Cargo.toml",
        "//proxydetox:Cargo.toml",
        "//proxydetoxlib:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_repositories")

rust_bindgen_repositories()

versioned_http_archive(
    name = "rules_pkg",
    sha256 = "eea0f59c28a9241156a47d7a8e32db9122f3d50b505fae0f33de6ce4d9b61834",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
    ],
    version = "0.8.1",
)

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()
