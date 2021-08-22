workspace(name = "proxydetox")

load("//third_party:crate_universe_defaults.bzl", "DEFAULT_SHA256_CHECKSUMS", "DEFAULT_URL_TEMPLATE")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

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

http_archive(
    name = "rules_rust",
    sha256 = "5ef76c7ca318f0795c2a524a01e0a7a399fd845d7cdebf7bc7ea7321859069b6",
    strip_prefix = "rules_rust-af2f908a2d342d79b74ea97fcbfbe7b0d03e2bdf",
    urls = [
        # `main` branch as of 2021-08-17
        "https://github.com/bazelbuild/rules_rust/archive/af2f908a2d342d79b74ea97fcbfbe7b0d03e2bdf.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2018",
    version = "1.54.0",
)

load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_repositories")

rust_bindgen_repositories()

# load("//cargo:crates.bzl", "raze_fetch_remote_crates")

# raze_fetch_remote_crates()

load("@rules_rust//crate_universe:defs.bzl", "crate", "crate_universe")

crate_universe(
    name = "crates",
    cargo_toml_files = [
        "//libsspi:Cargo.toml",
        "//proxydetox:Cargo.toml",
        "//proxy_client:Cargo.toml",
        "//cproxydetox:Cargo.toml",
        "//duktape-sys:Cargo.toml",
        "//libnegotiate:Cargo.toml",
        "//paclib:Cargo.toml",
        "//paceval:Cargo.toml",
        "//duktape:Cargo.toml",
    ],
    # [package.metadata.raze.xxx] lines in Cargo.toml files are ignored;
    # the overrides need to be declared in the repo rule instead.
    overrides = {
        "clang-sys": crate.override(
            extra_build_script_env_vars = {"PATH": "/var/empty"},
        ),
    },
    resolver_download_url_template = DEFAULT_URL_TEMPLATE,
    resolver_sha256s = DEFAULT_SHA256_CHECKSUMS,
    # leave unset for default multi-platform support
    # supported_targets = [
    #     "x86_64-apple-darwin",
    #     "x86_64-unknown-linux-gnu",
    # ],
    # to use a lockfile, uncomment the following line,
    # create an empty file in the location, and then build
    # with REPIN=1 bazel build ...
    #lockfile = "//:crate_universe.lock",
)

load("@crates//:defs.bzl", "pinned_rust_install")

pinned_rust_install()

http_archive(
    name = "rules_pkg",
    sha256 = "a89e203d3cf264e564fcb96b6e06dd70bc0557356eb48400ce4b5d97c2c3720d",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/0.5.1/rules_pkg-0.5.1.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/0.5.1/rules_pkg-0.5.1.tar.gz",
    ],
)

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()
