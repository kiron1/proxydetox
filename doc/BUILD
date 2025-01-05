load("@rules_rust_mdbook//:defs.bzl", "mdbook")

genrule(
    name = "proxydetox_help",
    outs = ["proxydetox_help.txt"],
    cmd = "$(location //proxydetox) --help > $@",
    tools = ["//proxydetox"],
)

mdbook(
    name = "doc",
    srcs = [
        "SUMMARY.md",
        "alternatives.md",
        "build.md",
        "cliref.md",
        "configuration.md",
        "cross_compile.md",
        "developers.md",
        "installation.md",
        "introduction.md",
        "service.md",
        "transparent_proxy.md",
        "usage.md",
        ":proxydetox_help",
    ],
    book = "//:book.toml",
    visibility = ["//www:__subpackages__"],
)
