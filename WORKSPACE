workspace(name = "proxydetox")

load("@//bazel:http.bzl", "versioned_http_archive")

versioned_http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "34c41bfb59cdaea29ac2df5a2fa79e5add609c71bb303b2ebb10985f93fa20e7",
    urls = [
        "https://github.com/bazelbuild/rules_apple/releases/download/{version}/rules_apple.{version}.tar.gz",
    ],
    version = "3.1.1",
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
    sha256 = "9ecd0f2144f0a24e6bc71ebcc50a1ee5128cedeceb32187004532c9710cb2334",
    urls = [
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz",
    ],
    version = "0.29.1",
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(edition = "2021")

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
        "//detox_net:Cargo.toml",
        "//dnsdetox:Cargo.toml",
        "//paceval:Cargo.toml",
        "//paclib:Cargo.toml",
        "//proxy_client:Cargo.toml",
        "//proxydetox:Cargo.toml",
        "//proxydetoxlib:Cargo.toml",
        "//spnego:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_dependencies", "rust_bindgen_register_toolchains")

rust_bindgen_dependencies()

rust_bindgen_register_toolchains()

load("@rules_rust//bindgen:transitive_repositories.bzl", "rust_bindgen_transitive_dependencies")

rust_bindgen_transitive_dependencies()

versioned_http_archive(
    name = "rules_pkg",
    sha256 = "8f9ee2dc10c1ae514ee599a8b42ed99fa262b757058f65ad3c384289ff70c4b8",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
    ],
    version = "0.9.1",
)

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()
