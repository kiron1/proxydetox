workspace(name = "proxydetox")

load("@//bazel:http.bzl", "versioned_http_archive")

# https://github.com/bazelbuild/platforms/releases
versioned_http_archive(
    name = "platforms",
    sha256 = "218efe8ee736d26a3572663b374a253c012b716d8af0c07e842e82f238a0a7ee",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/platforms/releases/download/{version}/platforms-{version}.tar.gz",
        "https://github.com/bazelbuild/platforms/releases/download/{version}/platforms-{version}.tar.gz",
    ],
    version = "0.0.10",
)

# https://github.com/bazelbuild/rules_cc/releases
versioned_http_archive(
    name = "rules_cc",
    sha256 = "d75a040c32954da0d308d3f2ea2ba735490f49b3a7aa3e4b40259ca4b814f825",
    # strip_prefix = "rules_cc-{version}",
    urls = ["https://github.com/bazelbuild/rules_cc/releases/download/{version}/rules_cc-{version}.tar.gz"],
    version = "0.0.10-rc1",
)

# https://github.com/bazelbuild/rules_rust/releases
versioned_http_archive(
    name = "rules_rust",
    # integrity = "sha256-+bWb47wg0VchIADaHt6L5Dma2Gn+Q589nz/MKcTi+lo=",
    integrity = "sha256-JLN47ZcAbx9wEr5Jiib4HduZATGLiDgK7oUi/fvotzU=",
    urls = [
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz",
    ],
    # version = "0.45.1",
    version = "0.42.1",
)

# https://github.com/bazelbuild/rules_apple/releases
versioned_http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "b4df908ec14868369021182ab191dbd1f40830c9b300650d5dc389e0b9266c8d",
    urls = [
        "https://github.com/bazelbuild/rules_apple/releases/download/{version}/rules_apple.{version}.tar.gz",
    ],
    version = "3.5.1",
)

# https://github.com/bazelbuild/apple_support/releases
versioned_http_archive(
    name = "build_bazel_apple_support",
    sha256 = "c4bb2b7367c484382300aee75be598b92f847896fb31bbd22f3a2346adf66a80",
    urls = [
        "https://github.com/bazelbuild/apple_support/releases/download/{version}/apple_support.{version}.tar.gz",
    ],
    version = "1.15.1",
)

# https://github.com/bazelbuild/rules_swift/releases
versioned_http_archive(
    name = "build_bazel_rules_swift",
    sha256 = "bb01097c7c7a1407f8ad49a1a0b1960655cf823c26ad2782d0b7d15b323838e2",
    urls = [
        "https://github.com/bazelbuild/rules_swift/releases/download/{version}/rules_swift.{version}.tar.gz",
    ],
    version = "1.18.0",
)

# https://github.com/bazelbuild/rules_pkg/releases
versioned_http_archive(
    name = "rules_pkg",
    sha256 = "d250924a2ecc5176808fc4c25d5cf5e9e79e6346d79d5ab1c493e289e722d1d0",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/{version}/rules_pkg-{version}.tar.gz",
    ],
    version = "0.10.1",
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
        "//detox_auth:Cargo.toml",
        "//detox_net:Cargo.toml",
        "//detox_futures:Cargo.toml",
        "//detox_hyper:Cargo.toml",
        "//dnsdetox:Cargo.toml",
        "//paceval:Cargo.toml",
        "//paclib:Cargo.toml",
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
