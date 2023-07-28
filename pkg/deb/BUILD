load("@rules_pkg//pkg:deb.bzl", "pkg_deb")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")

pkg_tar(
    name = "bin",
    srcs = ["//proxydetox"],
    package_dir = "usr/bin",
)

pkg_tar(
    name = "systemd_user",
    srcs = ["proxydetox.service"],
    package_dir = "lib/systemd/user",
)

pkg_tar(
    name = "data",
    deps = [
        ":bin",
        ":systemd_user",
    ],
)

pkg_deb(
    name = "deb",
    out = "proxydetox.deb",
    architecture = select({
        "@platforms//cpu:arm64": "arm64",
        "@platforms//cpu:x86_32": "i386",
        "@platforms//cpu:x86_64": "amd64",
    }),
    data = ":data",
    depends = [
        "libc6 (>= 2.17)",
        "libssl1.1 (>= 1.1.1)",
    ] + select({
        "//:enable_negotiate": ["libgssapi-krb5-2 (>= 1.17)"],
        "//conditions:default": [],
    }),
    description = "A proxy for localhost to simplify life with corperate proxies.",
    homepage = "https://proxydetox.colorto.cc/",
    maintainer = "Kiron <kiron1@gmail.com>",
    package = "proxydetox",
    package_file_name = "proxydetox_{version}_{architecture}.deb",
    package_variables = "//pkg:variables",
    postinst = ":postinst",
    postrm = ":postrm",
    section = "net",
    target_compatible_with = ["@platforms//os:linux"],
    version_file = "//pkg:version",
    visibility = ["//visibility:public"],
)
