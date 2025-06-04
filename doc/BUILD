load("@rules_rust_mdbook//:defs.bzl", "mdbook")

genrule(
    name = "proxydetox_help",
    outs = ["proxydetox_help.txt"],
    cmd = "PROXYDETOX_NORC=1 $(location //proxydetox) --help > $@",
    tools = ["//proxydetox"],
)

mdbook(
    name = "doc",
    srcs = [
        ":proxydetox_help",
        "SUMMARY.md",
        "alternatives.md",
        "build.md",
        "cliref.md",
        "configuration.md",
        "cross_compile.md",
        "developers.md",
        "installation.md",
        "introduction.md",
        "launchctl.sh",
        "service.md",
        "transparent_proxy.md",
        "usage.md",
        "windows.md",
    ],
    book = "//:book.toml",
    visibility = ["//www:__subpackages__"],
)
