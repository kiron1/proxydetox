load("@rules_pkg//:pkg.bzl", "pkg_tar")
load("@rules_shell//shell:sh_binary.bzl", "sh_binary")

pkg_tar(
    name = "www",
    srcs = [
        "bootstrap.min.css",
        "bootstrap.min.js",
        "cover.css",
        "index.html",
        "jquery-slim.min.js",
        "popper.min.js",
        "//doc",
    ],
    extension = ".tar.gz",
)

sh_binary(
    name = "copy",
    srcs = ["copy.sh"],
    data = [":www"],
    env = {"WWW_TARBALL": "$(location :www)"},
)
