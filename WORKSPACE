workspace(name = "proxydetox")

load("@//bazel:http.bzl", "versioned_http_archive")

# https://github.com/bazelbuild/platforms/releases
versioned_http_archive(
    name = "platforms",
    sha256 = "8150406605389ececb6da07cbcb509d5637a3ab9a24bc69b1101531367d89d74",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/platforms/releases/download/{version}/platforms-{version}.tar.gz",
        "https://github.com/bazelbuild/platforms/releases/download/{version}/platforms-{version}.tar.gz",
    ],
    version = "0.0.8",
)

# https://github.com/bazelbuild/rules_rust/releases
versioned_http_archive(
    name = "rules_rust",
    sha256 = "75177226380b771be36d7efc538da842c433f14cd6c36d7660976efb53defe86",
    urls = [
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz",
    ],
    version = "0.34.1",
)

# https://github.com/bazelbuild/rules_apple/releases
versioned_http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "34c41bfb59cdaea29ac2df5a2fa79e5add609c71bb303b2ebb10985f93fa20e7",
    urls = [
        "https://github.com/bazelbuild/rules_apple/releases/download/{version}/rules_apple.{version}.tar.gz",
    ],
    version = "3.1.1",
)

# https://github.com/bazelbuild/apple_support/releases
versioned_http_archive(
    name = "build_bazel_apple_support",
    sha256 = "cf4d63f39c7ba9059f70e995bf5fe1019267d3f77379c2028561a5d7645ef67c",
    urls = [
        "https://github.com/bazelbuild/apple_support/releases/download/{version}/apple_support.{version}.tar.gz",
    ],
    version = "1.11.1",
)

# https://github.com/bazelbuild/rules_swift/releases
versioned_http_archive(
    name = "build_bazel_rules_swift",
    sha256 = "28a66ff5d97500f0304f4e8945d936fe0584e0d5b7a6f83258298007a93190ba",
    urls = [
        "https://github.com/bazelbuild/rules_swift/releases/download/{version}/rules_swift.{version}.tar.gz",
    ],
    version = "1.13.0",
)

# https://github.com/bazelbuild/rules_pkg/releases
versioned_http_archive(
    name = "rules_pkg",
    sha256 = "8f9ee2dc10c1ae514ee599a8b42ed99fa262b757058f65ad3c384289ff70c4b8",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
    ],
    version = "0.9.1",
)

load(
    "@build_bazel_apple_support//lib:repositories.bzl",
    "apple_support_dependencies",
)

apple_support_dependencies()

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

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    extra_target_triples = [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
    ],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository")

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

crates_repository(
    name = "crate_index_tools",
    cargo_lockfile = "//:Cargo.tools.lock",
    # Generate with:
    # CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index_tools
    lockfile = "//:Cargo.Bazel.tools.lock",
    packages = {
        "clap": crate.spec(
            default_features = False,
            features = [
                "std",
                "derive",
            ],
            version = "4.3",
        ),
        "toml": crate.spec(version = "0.7.6"),
    },
)

load("@crate_index_tools//:defs.bzl", crate_repositories_tools = "crate_repositories")

crate_repositories_tools()

load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_dependencies", "rust_bindgen_register_toolchains")

rust_bindgen_dependencies()

rust_bindgen_register_toolchains()

load("@rules_rust//bindgen:transitive_repositories.bzl", "rust_bindgen_transitive_dependencies")

rust_bindgen_transitive_dependencies()

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()
