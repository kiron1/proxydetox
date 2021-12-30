"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")  # buildifier: disable=load

def raze_fetch_remote_crates():
    """This function defines a collection of repos and should be called in a WORKSPACE file"""
    maybe(
        http_archive,
        name = "raze__aho_corasick__0_7_18",
        url = "https://crates.io/api/v1/crates/aho-corasick/0.7.18/download",
        type = "tar.gz",
        sha256 = "1e37cfd5e7657ada45f742d6e99ca5788580b5c529dc78faf11ece6dc702656f",
        strip_prefix = "aho-corasick-0.7.18",
        build_file = Label("//cargo/remote:BUILD.aho-corasick-0.7.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ansi_term__0_12_1",
        url = "https://crates.io/api/v1/crates/ansi_term/0.12.1/download",
        type = "tar.gz",
        sha256 = "d52a9bb7ec0cf484c551830a7ce27bd20d67eac647e1befb56b0be4ee39a55d2",
        strip_prefix = "ansi_term-0.12.1",
        build_file = Label("//cargo/remote:BUILD.ansi_term-0.12.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__argh__0_1_7",
        url = "https://crates.io/api/v1/crates/argh/0.1.7/download",
        type = "tar.gz",
        sha256 = "dbb41d85d92dfab96cb95ab023c265c5e4261bb956c0fb49ca06d90c570f1958",
        strip_prefix = "argh-0.1.7",
        build_file = Label("//cargo/remote:BUILD.argh-0.1.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__argh_derive__0_1_7",
        url = "https://crates.io/api/v1/crates/argh_derive/0.1.7/download",
        type = "tar.gz",
        sha256 = "be69f70ef5497dd6ab331a50bd95c6ac6b8f7f17a7967838332743fbd58dc3b5",
        strip_prefix = "argh_derive-0.1.7",
        build_file = Label("//cargo/remote:BUILD.argh_derive-0.1.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__argh_shared__0_1_7",
        url = "https://crates.io/api/v1/crates/argh_shared/0.1.7/download",
        type = "tar.gz",
        sha256 = "e6f8c380fa28aa1b36107cd97f0196474bb7241bb95a453c5c01a15ac74b2eac",
        strip_prefix = "argh_shared-0.1.7",
        build_file = Label("//cargo/remote:BUILD.argh_shared-0.1.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__atty__0_2_14",
        url = "https://crates.io/api/v1/crates/atty/0.2.14/download",
        type = "tar.gz",
        sha256 = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8",
        strip_prefix = "atty-0.2.14",
        build_file = Label("//cargo/remote:BUILD.atty-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__autocfg__1_0_1",
        url = "https://crates.io/api/v1/crates/autocfg/1.0.1/download",
        type = "tar.gz",
        sha256 = "cdb031dd78e28731d87d56cc8ffef4a8f36ca26c38fe2de700543e627f8a464a",
        strip_prefix = "autocfg-1.0.1",
        build_file = Label("//cargo/remote:BUILD.autocfg-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__base64__0_13_0",
        url = "https://crates.io/api/v1/crates/base64/0.13.0/download",
        type = "tar.gz",
        sha256 = "904dfeac50f3cdaba28fc6f57fdcddb75f49ed61346676a78c4ffe55877802fd",
        strip_prefix = "base64-0.13.0",
        build_file = Label("//cargo/remote:BUILD.base64-0.13.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bindgen__0_59_2",
        url = "https://crates.io/api/v1/crates/bindgen/0.59.2/download",
        type = "tar.gz",
        sha256 = "2bd2a9a458e8f4304c52c43ebb0cfbd520289f8379a52e329a38afda99bf8eb8",
        strip_prefix = "bindgen-0.59.2",
        build_file = Label("//cargo/remote:BUILD.bindgen-0.59.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bitflags__1_3_2",
        url = "https://crates.io/api/v1/crates/bitflags/1.3.2/download",
        type = "tar.gz",
        sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a",
        strip_prefix = "bitflags-1.3.2",
        build_file = Label("//cargo/remote:BUILD.bitflags-1.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__byteorder__1_4_3",
        url = "https://crates.io/api/v1/crates/byteorder/1.4.3/download",
        type = "tar.gz",
        sha256 = "14c189c53d098945499cdfa7ecc63567cf3886b3332b312a5b4585d8d3a6a610",
        strip_prefix = "byteorder-1.4.3",
        build_file = Label("//cargo/remote:BUILD.byteorder-1.4.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bytes__1_1_0",
        url = "https://crates.io/api/v1/crates/bytes/1.1.0/download",
        type = "tar.gz",
        sha256 = "c4872d67bab6358e59559027aa3b9157c53d9358c51423c17554809a8858e0f8",
        strip_prefix = "bytes-1.1.0",
        build_file = Label("//cargo/remote:BUILD.bytes-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cc__1_0_72",
        url = "https://crates.io/api/v1/crates/cc/1.0.72/download",
        type = "tar.gz",
        sha256 = "22a9137b95ea06864e018375b72adfb7db6e6f68cfc8df5a04d00288050485ee",
        strip_prefix = "cc-1.0.72",
        build_file = Label("//cargo/remote:BUILD.cc-1.0.72.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cexpr__0_6_0",
        url = "https://crates.io/api/v1/crates/cexpr/0.6.0/download",
        type = "tar.gz",
        sha256 = "6fac387a98bb7c37292057cffc56d62ecb629900026402633ae9160df93a8766",
        strip_prefix = "cexpr-0.6.0",
        build_file = Label("//cargo/remote:BUILD.cexpr-0.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cfg_if__1_0_0",
        url = "https://crates.io/api/v1/crates/cfg-if/1.0.0/download",
        type = "tar.gz",
        sha256 = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd",
        strip_prefix = "cfg-if-1.0.0",
        build_file = Label("//cargo/remote:BUILD.cfg-if-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clang_sys__1_3_0",
        url = "https://crates.io/api/v1/crates/clang-sys/1.3.0/download",
        type = "tar.gz",
        sha256 = "fa66045b9cb23c2e9c1520732030608b02ee07e5cfaa5a521ec15ded7fa24c90",
        strip_prefix = "clang-sys-1.3.0",
        build_file = Label("//cargo/remote:BUILD.clang-sys-1.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap__2_34_0",
        url = "https://crates.io/api/v1/crates/clap/2.34.0/download",
        type = "tar.gz",
        sha256 = "a0610544180c38b88101fecf2dd634b174a62eef6946f84dfc6a7127512b381c",
        strip_prefix = "clap-2.34.0",
        build_file = Label("//cargo/remote:BUILD.clap-2.34.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation__0_9_2",
        url = "https://crates.io/api/v1/crates/core-foundation/0.9.2/download",
        type = "tar.gz",
        sha256 = "6888e10551bb93e424d8df1d07f1a8b4fceb0001a3a4b048bfc47554946f47b3",
        strip_prefix = "core-foundation-0.9.2",
        build_file = Label("//cargo/remote:BUILD.core-foundation-0.9.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation_sys__0_8_3",
        url = "https://crates.io/api/v1/crates/core-foundation-sys/0.8.3/download",
        type = "tar.gz",
        sha256 = "5827cebf4670468b8772dd191856768aedcb1b0278a04f989f7766351917b9dc",
        strip_prefix = "core-foundation-sys-0.8.3",
        build_file = Label("//cargo/remote:BUILD.core-foundation-sys-0.8.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dirs__4_0_0",
        url = "https://crates.io/api/v1/crates/dirs/4.0.0/download",
        type = "tar.gz",
        sha256 = "ca3aa72a6f96ea37bbc5aa912f6788242832f75369bdfdadcb0e38423f100059",
        strip_prefix = "dirs-4.0.0",
        build_file = Label("//cargo/remote:BUILD.dirs-4.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dirs_sys__0_3_6",
        url = "https://crates.io/api/v1/crates/dirs-sys/0.3.6/download",
        type = "tar.gz",
        sha256 = "03d86534ed367a67548dc68113a0f5db55432fdfbb6e6f9d77704397d95d5780",
        strip_prefix = "dirs-sys-0.3.6",
        build_file = Label("//cargo/remote:BUILD.dirs-sys-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dns_parser__0_8_0",
        url = "https://crates.io/api/v1/crates/dns-parser/0.8.0/download",
        type = "tar.gz",
        sha256 = "c4d33be9473d06f75f58220f71f7a9317aca647dc061dbd3c361b0bef505fbea",
        strip_prefix = "dns-parser-0.8.0",
        build_file = Label("//cargo/remote:BUILD.dns-parser-0.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__either__1_6_1",
        url = "https://crates.io/api/v1/crates/either/1.6.1/download",
        type = "tar.gz",
        sha256 = "e78d4f1cc4ae33bbfc157ed5d5a5ef3bc29227303d595861deb238fcec4e9457",
        strip_prefix = "either-1.6.1",
        build_file = Label("//cargo/remote:BUILD.either-1.6.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__env_logger__0_9_0",
        url = "https://crates.io/api/v1/crates/env_logger/0.9.0/download",
        type = "tar.gz",
        sha256 = "0b2cf0344971ee6c64c31be0d530793fba457d322dfec2810c453d0ef228f9c3",
        strip_prefix = "env_logger-0.9.0",
        build_file = Label("//cargo/remote:BUILD.env_logger-0.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fnv__1_0_7",
        url = "https://crates.io/api/v1/crates/fnv/1.0.7/download",
        type = "tar.gz",
        sha256 = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1",
        strip_prefix = "fnv-1.0.7",
        build_file = Label("//cargo/remote:BUILD.fnv-1.0.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types__0_3_2",
        url = "https://crates.io/api/v1/crates/foreign-types/0.3.2/download",
        type = "tar.gz",
        sha256 = "f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1",
        strip_prefix = "foreign-types-0.3.2",
        build_file = Label("//cargo/remote:BUILD.foreign-types-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types_shared__0_1_1",
        url = "https://crates.io/api/v1/crates/foreign-types-shared/0.1.1/download",
        type = "tar.gz",
        sha256 = "00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b",
        strip_prefix = "foreign-types-shared-0.1.1",
        build_file = Label("//cargo/remote:BUILD.foreign-types-shared-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures__0_3_19",
        url = "https://crates.io/api/v1/crates/futures/0.3.19/download",
        type = "tar.gz",
        sha256 = "28560757fe2bb34e79f907794bb6b22ae8b0e5c669b638a1132f2592b19035b4",
        strip_prefix = "futures-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_channel__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-channel/0.3.19/download",
        type = "tar.gz",
        sha256 = "ba3dda0b6588335f360afc675d0564c17a77a2bda81ca178a4b6081bd86c7f0b",
        strip_prefix = "futures-channel-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-channel-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_core__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-core/0.3.19/download",
        type = "tar.gz",
        sha256 = "d0c8ff0461b82559810cdccfde3215c3f373807f5e5232b71479bff7bb2583d7",
        strip_prefix = "futures-core-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-core-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_executor__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-executor/0.3.19/download",
        type = "tar.gz",
        sha256 = "29d6d2ff5bb10fb95c85b8ce46538a2e5f5e7fdc755623a7d4529ab8a4ed9d2a",
        strip_prefix = "futures-executor-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-executor-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_io__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-io/0.3.19/download",
        type = "tar.gz",
        sha256 = "b1f9d34af5a1aac6fb380f735fe510746c38067c5bf16c7fd250280503c971b2",
        strip_prefix = "futures-io-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-io-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_macro__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-macro/0.3.19/download",
        type = "tar.gz",
        sha256 = "6dbd947adfffb0efc70599b3ddcf7b5597bb5fa9e245eb99f62b3a5f7bb8bd3c",
        strip_prefix = "futures-macro-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-macro-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_sink__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-sink/0.3.19/download",
        type = "tar.gz",
        sha256 = "e3055baccb68d74ff6480350f8d6eb8fcfa3aa11bdc1a1ae3afdd0514617d508",
        strip_prefix = "futures-sink-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-sink-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_task__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-task/0.3.19/download",
        type = "tar.gz",
        sha256 = "6ee7c6485c30167ce4dfb83ac568a849fe53274c831081476ee13e0dce1aad72",
        strip_prefix = "futures-task-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-task-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_util__0_3_19",
        url = "https://crates.io/api/v1/crates/futures-util/0.3.19/download",
        type = "tar.gz",
        sha256 = "d9b5cf40b47a271f77a8b1bec03ca09044d99d2372c0de244e66430761127164",
        strip_prefix = "futures-util-0.3.19",
        build_file = Label("//cargo/remote:BUILD.futures-util-0.3.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__getrandom__0_2_3",
        url = "https://crates.io/api/v1/crates/getrandom/0.2.3/download",
        type = "tar.gz",
        sha256 = "7fcd999463524c52659517fe2cea98493cfe485d10565e7b0fb07dbba7ad2753",
        strip_prefix = "getrandom-0.2.3",
        build_file = Label("//cargo/remote:BUILD.getrandom-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__glob__0_3_0",
        url = "https://crates.io/api/v1/crates/glob/0.3.0/download",
        type = "tar.gz",
        sha256 = "9b919933a397b79c37e33b77bb2aa3dc8eb6e165ad809e58ff75bc7db2e34574",
        strip_prefix = "glob-0.3.0",
        build_file = Label("//cargo/remote:BUILD.glob-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__h2__0_3_9",
        url = "https://crates.io/api/v1/crates/h2/0.3.9/download",
        type = "tar.gz",
        sha256 = "8f072413d126e57991455e0a922b31e4c8ba7c2ffbebf6b78b4f8521397d65cd",
        strip_prefix = "h2-0.3.9",
        build_file = Label("//cargo/remote:BUILD.h2-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hashbrown__0_11_2",
        url = "https://crates.io/api/v1/crates/hashbrown/0.11.2/download",
        type = "tar.gz",
        sha256 = "ab5ef0d4909ef3724cc8cce6ccc8572c5c817592e9285f5464f8e86f8bd3726e",
        strip_prefix = "hashbrown-0.11.2",
        build_file = Label("//cargo/remote:BUILD.hashbrown-0.11.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__heck__0_3_3",
        url = "https://crates.io/api/v1/crates/heck/0.3.3/download",
        type = "tar.gz",
        sha256 = "6d621efb26863f0e9924c6ac577e8275e5e6b77455db64ffa6c65c904e9e132c",
        strip_prefix = "heck-0.3.3",
        build_file = Label("//cargo/remote:BUILD.heck-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hermit_abi__0_1_19",
        url = "https://crates.io/api/v1/crates/hermit-abi/0.1.19/download",
        type = "tar.gz",
        sha256 = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33",
        strip_prefix = "hermit-abi-0.1.19",
        build_file = Label("//cargo/remote:BUILD.hermit-abi-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http__0_2_6",
        url = "https://crates.io/api/v1/crates/http/0.2.6/download",
        type = "tar.gz",
        sha256 = "31f4c6746584866f0feabcc69893c5b51beef3831656a968ed7ae254cdc4fd03",
        strip_prefix = "http-0.2.6",
        build_file = Label("//cargo/remote:BUILD.http-0.2.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http_body__0_4_4",
        url = "https://crates.io/api/v1/crates/http-body/0.4.4/download",
        type = "tar.gz",
        sha256 = "1ff4f84919677303da5f147645dbea6b1881f368d03ac84e1dc09031ebd7b2c6",
        strip_prefix = "http-body-0.4.4",
        build_file = Label("//cargo/remote:BUILD.http-body-0.4.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__httparse__1_5_1",
        url = "https://crates.io/api/v1/crates/httparse/1.5.1/download",
        type = "tar.gz",
        sha256 = "acd94fdbe1d4ff688b67b04eee2e17bd50995534a61539e45adfefb45e5e5503",
        strip_prefix = "httparse-1.5.1",
        build_file = Label("//cargo/remote:BUILD.httparse-1.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__httpdate__1_0_2",
        url = "https://crates.io/api/v1/crates/httpdate/1.0.2/download",
        type = "tar.gz",
        sha256 = "c4a1e36c821dbe04574f602848a19f742f4fb3c98d40449f11bcad18d6b17421",
        strip_prefix = "httpdate-1.0.2",
        build_file = Label("//cargo/remote:BUILD.httpdate-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__humantime__2_1_0",
        url = "https://crates.io/api/v1/crates/humantime/2.1.0/download",
        type = "tar.gz",
        sha256 = "9a3a5bfb195931eeb336b2a7b4d761daec841b97f947d34394601737a7bba5e4",
        strip_prefix = "humantime-2.1.0",
        build_file = Label("//cargo/remote:BUILD.humantime-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper__0_14_16",
        url = "https://crates.io/api/v1/crates/hyper/0.14.16/download",
        type = "tar.gz",
        sha256 = "b7ec3e62bdc98a2f0393a5048e4c30ef659440ea6e0e572965103e72bd836f55",
        strip_prefix = "hyper-0.14.16",
        build_file = Label("//cargo/remote:BUILD.hyper-0.14.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper_tls__0_5_0",
        url = "https://crates.io/api/v1/crates/hyper-tls/0.5.0/download",
        type = "tar.gz",
        sha256 = "d6183ddfa99b85da61a140bea0efc93fdf56ceaa041b37d553518030827f9905",
        strip_prefix = "hyper-tls-0.5.0",
        build_file = Label("//cargo/remote:BUILD.hyper-tls-0.5.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indexmap__1_7_0",
        url = "https://crates.io/api/v1/crates/indexmap/1.7.0/download",
        type = "tar.gz",
        sha256 = "bc633605454125dec4b66843673f01c7df2b89479b32e0ed634e43a91cff62a5",
        strip_prefix = "indexmap-1.7.0",
        build_file = Label("//cargo/remote:BUILD.indexmap-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__inotify__0_10_0",
        url = "https://crates.io/api/v1/crates/inotify/0.10.0/download",
        type = "tar.gz",
        sha256 = "abf888f9575c290197b2c948dc9e9ff10bd1a39ad1ea8585f734585fa6b9d3f9",
        strip_prefix = "inotify-0.10.0",
        build_file = Label("//cargo/remote:BUILD.inotify-0.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__inotify_sys__0_1_5",
        url = "https://crates.io/api/v1/crates/inotify-sys/0.1.5/download",
        type = "tar.gz",
        sha256 = "e05c02b5e89bff3b946cedeca278abc628fe811e604f027c45a8aa3cf793d0eb",
        strip_prefix = "inotify-sys-0.1.5",
        build_file = Label("//cargo/remote:BUILD.inotify-sys-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__0_4_8",
        url = "https://crates.io/api/v1/crates/itoa/0.4.8/download",
        type = "tar.gz",
        sha256 = "b71991ff56294aa922b450139ee08b3bfc70982c6b2c7562771375cf73542dd4",
        strip_prefix = "itoa-0.4.8",
        build_file = Label("//cargo/remote:BUILD.itoa-0.4.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__1_0_1",
        url = "https://crates.io/api/v1/crates/itoa/1.0.1/download",
        type = "tar.gz",
        sha256 = "1aab8fc367588b89dcee83ab0fd66b72b50b72fa1904d7095045ace2b0c81c35",
        strip_prefix = "itoa-1.0.1",
        build_file = Label("//cargo/remote:BUILD.itoa-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazy_static__1_4_0",
        url = "https://crates.io/api/v1/crates/lazy_static/1.4.0/download",
        type = "tar.gz",
        sha256 = "e2abad23fbc42b3700f2f279844dc832adb2b2eb069b2df918f455c4e18cc646",
        strip_prefix = "lazy_static-1.4.0",
        build_file = Label("//cargo/remote:BUILD.lazy_static-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazycell__1_3_0",
        url = "https://crates.io/api/v1/crates/lazycell/1.3.0/download",
        type = "tar.gz",
        sha256 = "830d08ce1d1d941e6b30645f1a0eb5643013d835ce3779a5fc208261dbe10f55",
        strip_prefix = "lazycell-1.3.0",
        build_file = Label("//cargo/remote:BUILD.lazycell-1.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libc__0_2_112",
        url = "https://crates.io/api/v1/crates/libc/0.2.112/download",
        type = "tar.gz",
        sha256 = "1b03d17f364a3a042d5e5d46b053bbbf82c92c9430c592dd4c064dc6ee997125",
        strip_prefix = "libc-0.2.112",
        build_file = Label("//cargo/remote:BUILD.libc-0.2.112.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libloading__0_7_2",
        url = "https://crates.io/api/v1/crates/libloading/0.7.2/download",
        type = "tar.gz",
        sha256 = "afe203d669ec979b7128619bae5a63b7b42e9203c1b29146079ee05e2f604b52",
        strip_prefix = "libloading-0.7.2",
        build_file = Label("//cargo/remote:BUILD.libloading-0.7.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log__0_4_14",
        url = "https://crates.io/api/v1/crates/log/0.4.14/download",
        type = "tar.gz",
        sha256 = "51b9bbe6c47d51fc3e1a9b945965946b4c44142ab8792c50835a980d362c2710",
        strip_prefix = "log-0.4.14",
        build_file = Label("//cargo/remote:BUILD.log-0.4.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__2_4_1",
        url = "https://crates.io/api/v1/crates/memchr/2.4.1/download",
        type = "tar.gz",
        sha256 = "308cc39be01b73d0d18f82a0e7b2a3df85245f84af96fdddc5d202d27e47b86a",
        strip_prefix = "memchr-2.4.1",
        build_file = Label("//cargo/remote:BUILD.memchr-2.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__minimal_lexical__0_2_1",
        url = "https://crates.io/api/v1/crates/minimal-lexical/0.2.1/download",
        type = "tar.gz",
        sha256 = "68354c5c6bd36d73ff3feceb05efa59b6acb7626617f4962be322a825e61f79a",
        strip_prefix = "minimal-lexical-0.2.1",
        build_file = Label("//cargo/remote:BUILD.minimal-lexical-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mio__0_7_14",
        url = "https://crates.io/api/v1/crates/mio/0.7.14/download",
        type = "tar.gz",
        sha256 = "8067b404fe97c70829f082dec8bcf4f71225d7eaea1d8645349cb76fa06205cc",
        strip_prefix = "mio-0.7.14",
        build_file = Label("//cargo/remote:BUILD.mio-0.7.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miow__0_3_7",
        url = "https://crates.io/api/v1/crates/miow/0.3.7/download",
        type = "tar.gz",
        sha256 = "b9f1c5b025cda876f66ef43a113f91ebc9f4ccef34843000e0adf6ebbab84e21",
        strip_prefix = "miow-0.3.7",
        build_file = Label("//cargo/remote:BUILD.miow-0.3.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__native_tls__0_2_8",
        url = "https://crates.io/api/v1/crates/native-tls/0.2.8/download",
        type = "tar.gz",
        sha256 = "48ba9f7719b5a0f42f338907614285fb5fd70e53858141f69898a1fb7203b24d",
        strip_prefix = "native-tls-0.2.8",
        build_file = Label("//cargo/remote:BUILD.native-tls-0.2.8.bazel"),
    )

    maybe(
        new_git_repository,
        name = "raze__netrc__0_4_1",
        remote = "https://github.com/kiron1/netrc-rs",
        commit = "82aa95f8d43b480f9ddb1695f0e4d73d6e168cb5",
        build_file = Label("//cargo/remote:BUILD.netrc-0.4.1.bazel"),
        init_submodules = True,
    )

    maybe(
        http_archive,
        name = "raze__nom__7_1_0",
        url = "https://crates.io/api/v1/crates/nom/7.1.0/download",
        type = "tar.gz",
        sha256 = "1b1d11e1ef389c76fe5b81bcaf2ea32cf88b62bc494e19f493d0b30e7a930109",
        strip_prefix = "nom-7.1.0",
        build_file = Label("//cargo/remote:BUILD.nom-7.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ntapi__0_3_6",
        url = "https://crates.io/api/v1/crates/ntapi/0.3.6/download",
        type = "tar.gz",
        sha256 = "3f6bb902e437b6d86e03cce10a7e2af662292c5dfef23b65899ea3ac9354ad44",
        strip_prefix = "ntapi-0.3.6",
        build_file = Label("//cargo/remote:BUILD.ntapi-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_cpus__1_13_1",
        url = "https://crates.io/api/v1/crates/num_cpus/1.13.1/download",
        type = "tar.gz",
        sha256 = "19e64526ebdee182341572e50e9ad03965aa510cd94427a4549448f285e957a1",
        strip_prefix = "num_cpus-1.13.1",
        build_file = Label("//cargo/remote:BUILD.num_cpus-1.13.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__once_cell__1_9_0",
        url = "https://crates.io/api/v1/crates/once_cell/1.9.0/download",
        type = "tar.gz",
        sha256 = "da32515d9f6e6e489d7bc9d84c71b060db7247dc035bbe44eac88cf87486d8d5",
        strip_prefix = "once_cell-1.9.0",
        build_file = Label("//cargo/remote:BUILD.once_cell-1.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl__0_10_38",
        url = "https://crates.io/api/v1/crates/openssl/0.10.38/download",
        type = "tar.gz",
        sha256 = "0c7ae222234c30df141154f159066c5093ff73b63204dcda7121eb082fc56a95",
        strip_prefix = "openssl-0.10.38",
        build_file = Label("//cargo/remote:BUILD.openssl-0.10.38.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_probe__0_1_4",
        url = "https://crates.io/api/v1/crates/openssl-probe/0.1.4/download",
        type = "tar.gz",
        sha256 = "28988d872ab76095a6e6ac88d99b54fd267702734fd7ffe610ca27f533ddb95a",
        strip_prefix = "openssl-probe-0.1.4",
        build_file = Label("//cargo/remote:BUILD.openssl-probe-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_sys__0_9_72",
        url = "https://crates.io/api/v1/crates/openssl-sys/0.9.72/download",
        type = "tar.gz",
        sha256 = "7e46109c383602735fa0a2e48dd2b7c892b048e1bf69e5c3b1d804b7d9c203cb",
        strip_prefix = "openssl-sys-0.9.72",
        build_file = Label("//cargo/remote:BUILD.openssl-sys-0.9.72.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__peeking_take_while__0_1_2",
        url = "https://crates.io/api/v1/crates/peeking_take_while/0.1.2/download",
        type = "tar.gz",
        sha256 = "19b17cddbe7ec3f8bc800887bab5e717348c95ea2ca0b1bf0837fb964dc67099",
        strip_prefix = "peeking_take_while-0.1.2",
        build_file = Label("//cargo/remote:BUILD.peeking_take_while-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_project__1_0_9",
        url = "https://crates.io/api/v1/crates/pin-project/1.0.9/download",
        type = "tar.gz",
        sha256 = "1622113ce508488160cff04e6abc60960e676d330e1ca0f77c0b8df17c81438f",
        strip_prefix = "pin-project-1.0.9",
        build_file = Label("//cargo/remote:BUILD.pin-project-1.0.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_project_internal__1_0_9",
        url = "https://crates.io/api/v1/crates/pin-project-internal/1.0.9/download",
        type = "tar.gz",
        sha256 = "b95af56fee93df76d721d356ac1ca41fccf168bc448eb14049234df764ba3e76",
        strip_prefix = "pin-project-internal-1.0.9",
        build_file = Label("//cargo/remote:BUILD.pin-project-internal-1.0.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_project_lite__0_2_7",
        url = "https://crates.io/api/v1/crates/pin-project-lite/0.2.7/download",
        type = "tar.gz",
        sha256 = "8d31d11c69a6b52a174b42bdc0c30e5e11670f90788b2c471c31c1d17d449443",
        strip_prefix = "pin-project-lite-0.2.7",
        build_file = Label("//cargo/remote:BUILD.pin-project-lite-0.2.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_utils__0_1_0",
        url = "https://crates.io/api/v1/crates/pin-utils/0.1.0/download",
        type = "tar.gz",
        sha256 = "8b870d8c151b6f2fb93e84a13146138f05d02ed11c7e7c54f8826aaaf7c9f184",
        strip_prefix = "pin-utils-0.1.0",
        build_file = Label("//cargo/remote:BUILD.pin-utils-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pkg_config__0_3_24",
        url = "https://crates.io/api/v1/crates/pkg-config/0.3.24/download",
        type = "tar.gz",
        sha256 = "58893f751c9b0412871a09abd62ecd2a00298c6c83befa223ef98c52aef40cbe",
        strip_prefix = "pkg-config-0.3.24",
        build_file = Label("//cargo/remote:BUILD.pkg-config-0.3.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ppv_lite86__0_2_16",
        url = "https://crates.io/api/v1/crates/ppv-lite86/0.2.16/download",
        type = "tar.gz",
        sha256 = "eb9f9e6e233e5c4a35559a617bf40a4ec447db2e84c20b55a6f83167b7e57872",
        strip_prefix = "ppv-lite86-0.2.16",
        build_file = Label("//cargo/remote:BUILD.ppv-lite86-0.2.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_36",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.36/download",
        type = "tar.gz",
        sha256 = "c7342d5883fbccae1cc37a2353b09c87c9b0f3afd73f5fb9bba687a1f733b029",
        strip_prefix = "proc-macro2-1.0.36",
        build_file = Label("//cargo/remote:BUILD.proc-macro2-1.0.36.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quick_error__1_2_3",
        url = "https://crates.io/api/v1/crates/quick-error/1.2.3/download",
        type = "tar.gz",
        sha256 = "a1d01941d82fa2ab50be1e79e6714289dd7cde78eba4c074bc5a4374f650dfe0",
        strip_prefix = "quick-error-1.2.3",
        build_file = Label("//cargo/remote:BUILD.quick-error-1.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_14",
        url = "https://crates.io/api/v1/crates/quote/1.0.14/download",
        type = "tar.gz",
        sha256 = "47aa80447ce4daf1717500037052af176af5d38cc3e571d9ec1c7353fc10c87d",
        strip_prefix = "quote-1.0.14",
        build_file = Label("//cargo/remote:BUILD.quote-1.0.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_8_4",
        url = "https://crates.io/api/v1/crates/rand/0.8.4/download",
        type = "tar.gz",
        sha256 = "2e7573632e6454cf6b99d7aac4ccca54be06da05aca2ef7423d22d27d4d4bcd8",
        strip_prefix = "rand-0.8.4",
        build_file = Label("//cargo/remote:BUILD.rand-0.8.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_chacha__0_3_1",
        url = "https://crates.io/api/v1/crates/rand_chacha/0.3.1/download",
        type = "tar.gz",
        sha256 = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88",
        strip_prefix = "rand_chacha-0.3.1",
        build_file = Label("//cargo/remote:BUILD.rand_chacha-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_core__0_6_3",
        url = "https://crates.io/api/v1/crates/rand_core/0.6.3/download",
        type = "tar.gz",
        sha256 = "d34f1408f55294453790c48b2f1ebbb1c5b4b7563eb1f418bcfcfdbb06ebb4e7",
        strip_prefix = "rand_core-0.6.3",
        build_file = Label("//cargo/remote:BUILD.rand_core-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_hc__0_3_1",
        url = "https://crates.io/api/v1/crates/rand_hc/0.3.1/download",
        type = "tar.gz",
        sha256 = "d51e9f596de227fda2ea6c84607f5558e196eeaf43c986b724ba4fb8fdf497e7",
        strip_prefix = "rand_hc-0.3.1",
        build_file = Label("//cargo/remote:BUILD.rand_hc-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_syscall__0_2_10",
        url = "https://crates.io/api/v1/crates/redox_syscall/0.2.10/download",
        type = "tar.gz",
        sha256 = "8383f39639269cde97d255a32bdb68c047337295414940c68bdd30c2e13203ff",
        strip_prefix = "redox_syscall-0.2.10",
        build_file = Label("//cargo/remote:BUILD.redox_syscall-0.2.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_users__0_4_0",
        url = "https://crates.io/api/v1/crates/redox_users/0.4.0/download",
        type = "tar.gz",
        sha256 = "528532f3d801c87aec9def2add9ca802fe569e44a544afe633765267840abe64",
        strip_prefix = "redox_users-0.4.0",
        build_file = Label("//cargo/remote:BUILD.redox_users-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex__1_5_4",
        url = "https://crates.io/api/v1/crates/regex/1.5.4/download",
        type = "tar.gz",
        sha256 = "d07a8629359eb56f1e2fb1652bb04212c072a87ba68546a04065d525673ac461",
        strip_prefix = "regex-1.5.4",
        build_file = Label("//cargo/remote:BUILD.regex-1.5.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_syntax__0_6_25",
        url = "https://crates.io/api/v1/crates/regex-syntax/0.6.25/download",
        type = "tar.gz",
        sha256 = "f497285884f3fcff424ffc933e56d7cbca511def0c9831a7f9b5f6153e3cc89b",
        strip_prefix = "regex-syntax-0.6.25",
        build_file = Label("//cargo/remote:BUILD.regex-syntax-0.6.25.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__remove_dir_all__0_5_3",
        url = "https://crates.io/api/v1/crates/remove_dir_all/0.5.3/download",
        type = "tar.gz",
        sha256 = "3acd125665422973a33ac9d3dd2df85edad0f4ae9b00dafb1a05e43a9f5ef8e7",
        strip_prefix = "remove_dir_all-0.5.3",
        build_file = Label("//cargo/remote:BUILD.remove_dir_all-0.5.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rustc_hash__1_1_0",
        url = "https://crates.io/api/v1/crates/rustc-hash/1.1.0/download",
        type = "tar.gz",
        sha256 = "08d43f7aa6b08d49f382cde6a7982047c3426db949b1424bc4b7ec9ae12c6ce2",
        strip_prefix = "rustc-hash-1.1.0",
        build_file = Label("//cargo/remote:BUILD.rustc-hash-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__schannel__0_1_19",
        url = "https://crates.io/api/v1/crates/schannel/0.1.19/download",
        type = "tar.gz",
        sha256 = "8f05ba609c234e60bee0d547fe94a4c7e9da733d1c962cf6e59efa4cd9c8bc75",
        strip_prefix = "schannel-0.1.19",
        build_file = Label("//cargo/remote:BUILD.schannel-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework__2_4_2",
        url = "https://crates.io/api/v1/crates/security-framework/2.4.2/download",
        type = "tar.gz",
        sha256 = "525bc1abfda2e1998d152c45cf13e696f76d0a4972310b22fac1658b05df7c87",
        strip_prefix = "security-framework-2.4.2",
        build_file = Label("//cargo/remote:BUILD.security-framework-2.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework_sys__2_4_2",
        url = "https://crates.io/api/v1/crates/security-framework-sys/2.4.2/download",
        type = "tar.gz",
        sha256 = "a9dd14d83160b528b7bfd66439110573efcfbe281b17fc2ca9f39f550d619c7e",
        strip_prefix = "security-framework-sys-2.4.2",
        build_file = Label("//cargo/remote:BUILD.security-framework-sys-2.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sharded_slab__0_1_4",
        url = "https://crates.io/api/v1/crates/sharded-slab/0.1.4/download",
        type = "tar.gz",
        sha256 = "900fba806f70c630b0a382d0d825e17a0f19fcd059a2ade1ff237bcddf446b31",
        strip_prefix = "sharded-slab-0.1.4",
        build_file = Label("//cargo/remote:BUILD.sharded-slab-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__shlex__1_1_0",
        url = "https://crates.io/api/v1/crates/shlex/1.1.0/download",
        type = "tar.gz",
        sha256 = "43b2853a4d09f215c24cc5489c992ce46052d359b5109343cbafbf26bc62f8a3",
        strip_prefix = "shlex-1.1.0",
        build_file = Label("//cargo/remote:BUILD.shlex-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__signal_hook_registry__1_4_0",
        url = "https://crates.io/api/v1/crates/signal-hook-registry/1.4.0/download",
        type = "tar.gz",
        sha256 = "e51e73328dc4ac0c7ccbda3a494dfa03df1de2f46018127f60c693f2648455b0",
        strip_prefix = "signal-hook-registry-1.4.0",
        build_file = Label("//cargo/remote:BUILD.signal-hook-registry-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__slab__0_4_5",
        url = "https://crates.io/api/v1/crates/slab/0.4.5/download",
        type = "tar.gz",
        sha256 = "9def91fd1e018fe007022791f865d0ccc9b3a0d5001e01aabb8b40e46000afb5",
        strip_prefix = "slab-0.4.5",
        build_file = Label("//cargo/remote:BUILD.slab-0.4.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__smallvec__1_7_0",
        url = "https://crates.io/api/v1/crates/smallvec/1.7.0/download",
        type = "tar.gz",
        sha256 = "1ecab6c735a6bb4139c0caafd0cc3635748bbb3acf4550e8138122099251f309",
        strip_prefix = "smallvec-1.7.0",
        build_file = Label("//cargo/remote:BUILD.smallvec-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__socket2__0_4_2",
        url = "https://crates.io/api/v1/crates/socket2/0.4.2/download",
        type = "tar.gz",
        sha256 = "5dc90fe6c7be1a323296982db1836d1ea9e47b6839496dde9a541bc496df3516",
        strip_prefix = "socket2-0.4.2",
        build_file = Label("//cargo/remote:BUILD.socket2-0.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__strsim__0_8_0",
        url = "https://crates.io/api/v1/crates/strsim/0.8.0/download",
        type = "tar.gz",
        sha256 = "8ea5119cdb4c55b55d432abb513a0429384878c15dde60cc77b1c99de1a95a6a",
        strip_prefix = "strsim-0.8.0",
        build_file = Label("//cargo/remote:BUILD.strsim-0.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__1_0_84",
        url = "https://crates.io/api/v1/crates/syn/1.0.84/download",
        type = "tar.gz",
        sha256 = "ecb2e6da8ee5eb9a61068762a32fa9619cc591ceb055b3687f4cd4051ec2e06b",
        strip_prefix = "syn-1.0.84",
        build_file = Label("//cargo/remote:BUILD.syn-1.0.84.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tempfile__3_2_0",
        url = "https://crates.io/api/v1/crates/tempfile/3.2.0/download",
        type = "tar.gz",
        sha256 = "dac1c663cfc93810f88aed9b8941d48cabf856a1b111c29a40439018d870eb22",
        strip_prefix = "tempfile-3.2.0",
        build_file = Label("//cargo/remote:BUILD.tempfile-3.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termcolor__1_1_2",
        url = "https://crates.io/api/v1/crates/termcolor/1.1.2/download",
        type = "tar.gz",
        sha256 = "2dfed899f0eb03f32ee8c6a0aabdb8a7949659e3466561fc0adf54e26d88c5f4",
        strip_prefix = "termcolor-1.1.2",
        build_file = Label("//cargo/remote:BUILD.termcolor-1.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__textwrap__0_11_0",
        url = "https://crates.io/api/v1/crates/textwrap/0.11.0/download",
        type = "tar.gz",
        sha256 = "d326610f408c7a4eb6f51c37c330e496b08506c9457c9d34287ecc38809fb060",
        strip_prefix = "textwrap-0.11.0",
        build_file = Label("//cargo/remote:BUILD.textwrap-0.11.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror__1_0_30",
        url = "https://crates.io/api/v1/crates/thiserror/1.0.30/download",
        type = "tar.gz",
        sha256 = "854babe52e4df1653706b98fcfc05843010039b406875930a70e4d9644e5c417",
        strip_prefix = "thiserror-1.0.30",
        build_file = Label("//cargo/remote:BUILD.thiserror-1.0.30.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror_impl__1_0_30",
        url = "https://crates.io/api/v1/crates/thiserror-impl/1.0.30/download",
        type = "tar.gz",
        sha256 = "aa32fd3f627f367fe16f893e2597ae3c05020f8bba2666a4e6ea73d377e5714b",
        strip_prefix = "thiserror-impl-1.0.30",
        build_file = Label("//cargo/remote:BUILD.thiserror-impl-1.0.30.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thread_local__1_1_3",
        url = "https://crates.io/api/v1/crates/thread_local/1.1.3/download",
        type = "tar.gz",
        sha256 = "8018d24e04c95ac8790716a5987d0fec4f8b27249ffa0f7d33f1369bdfb88cbd",
        strip_prefix = "thread_local-1.1.3",
        build_file = Label("//cargo/remote:BUILD.thread_local-1.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio__1_15_0",
        url = "https://crates.io/api/v1/crates/tokio/1.15.0/download",
        type = "tar.gz",
        sha256 = "fbbf1c778ec206785635ce8ad57fe52b3009ae9e0c9f574a728f3049d3e55838",
        strip_prefix = "tokio-1.15.0",
        build_file = Label("//cargo/remote:BUILD.tokio-1.15.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_macros__1_7_0",
        url = "https://crates.io/api/v1/crates/tokio-macros/1.7.0/download",
        type = "tar.gz",
        sha256 = "b557f72f448c511a979e2564e55d74e6c4432fc96ff4f6241bc6bded342643b7",
        strip_prefix = "tokio-macros-1.7.0",
        build_file = Label("//cargo/remote:BUILD.tokio-macros-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_native_tls__0_3_0",
        url = "https://crates.io/api/v1/crates/tokio-native-tls/0.3.0/download",
        type = "tar.gz",
        sha256 = "f7d995660bd2b7f8c1568414c1126076c13fbb725c40112dc0120b78eb9b717b",
        strip_prefix = "tokio-native-tls-0.3.0",
        build_file = Label("//cargo/remote:BUILD.tokio-native-tls-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_util__0_6_9",
        url = "https://crates.io/api/v1/crates/tokio-util/0.6.9/download",
        type = "tar.gz",
        sha256 = "9e99e1983e5d376cd8eb4b66604d2e99e79f5bd988c3055891dcd8c9e2604cc0",
        strip_prefix = "tokio-util-0.6.9",
        build_file = Label("//cargo/remote:BUILD.tokio-util-0.6.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tower_service__0_3_1",
        url = "https://crates.io/api/v1/crates/tower-service/0.3.1/download",
        type = "tar.gz",
        sha256 = "360dfd1d6d30e05fda32ace2c8c70e9c0a9da713275777f5a4dbb8a1893930c6",
        strip_prefix = "tower-service-0.3.1",
        build_file = Label("//cargo/remote:BUILD.tower-service-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing__0_1_29",
        url = "https://crates.io/api/v1/crates/tracing/0.1.29/download",
        type = "tar.gz",
        sha256 = "375a639232caf30edfc78e8d89b2d4c375515393e7af7e16f01cd96917fb2105",
        strip_prefix = "tracing-0.1.29",
        build_file = Label("//cargo/remote:BUILD.tracing-0.1.29.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_attributes__0_1_18",
        url = "https://crates.io/api/v1/crates/tracing-attributes/0.1.18/download",
        type = "tar.gz",
        sha256 = "f4f480b8f81512e825f337ad51e94c1eb5d3bbdf2b363dcd01e2b19a9ffe3f8e",
        strip_prefix = "tracing-attributes-0.1.18",
        build_file = Label("//cargo/remote:BUILD.tracing-attributes-0.1.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_core__0_1_21",
        url = "https://crates.io/api/v1/crates/tracing-core/0.1.21/download",
        type = "tar.gz",
        sha256 = "1f4ed65637b8390770814083d20756f87bfa2c21bf2f110babdc5438351746e4",
        strip_prefix = "tracing-core-0.1.21",
        build_file = Label("//cargo/remote:BUILD.tracing-core-0.1.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_futures__0_2_5",
        url = "https://crates.io/api/v1/crates/tracing-futures/0.2.5/download",
        type = "tar.gz",
        sha256 = "97d095ae15e245a057c8e8451bab9b3ee1e1f68e9ba2b4fbc18d0ac5237835f2",
        strip_prefix = "tracing-futures-0.2.5",
        build_file = Label("//cargo/remote:BUILD.tracing-futures-0.2.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_log__0_1_2",
        url = "https://crates.io/api/v1/crates/tracing-log/0.1.2/download",
        type = "tar.gz",
        sha256 = "a6923477a48e41c1951f1999ef8bb5a3023eb723ceadafe78ffb65dc366761e3",
        strip_prefix = "tracing-log-0.1.2",
        build_file = Label("//cargo/remote:BUILD.tracing-log-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_subscriber__0_3_5",
        url = "https://crates.io/api/v1/crates/tracing-subscriber/0.3.5/download",
        type = "tar.gz",
        sha256 = "5d81bfa81424cc98cb034b837c985b7a290f592e5b4322f353f94a0ab0f9f594",
        strip_prefix = "tracing-subscriber-0.3.5",
        build_file = Label("//cargo/remote:BUILD.tracing-subscriber-0.3.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__try_lock__0_2_3",
        url = "https://crates.io/api/v1/crates/try-lock/0.2.3/download",
        type = "tar.gz",
        sha256 = "59547bce71d9c38b83d9c0e92b6066c4253371f15005def0c30d9657f50c7642",
        strip_prefix = "try-lock-0.2.3",
        build_file = Label("//cargo/remote:BUILD.try-lock-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_segmentation__1_8_0",
        url = "https://crates.io/api/v1/crates/unicode-segmentation/1.8.0/download",
        type = "tar.gz",
        sha256 = "8895849a949e7845e06bd6dc1aa51731a103c42707010a5b591c0038fb73385b",
        strip_prefix = "unicode-segmentation-1.8.0",
        build_file = Label("//cargo/remote:BUILD.unicode-segmentation-1.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_width__0_1_9",
        url = "https://crates.io/api/v1/crates/unicode-width/0.1.9/download",
        type = "tar.gz",
        sha256 = "3ed742d4ea2bd1176e236172c8429aaf54486e7ac098db29ffe6529e0ce50973",
        strip_prefix = "unicode-width-0.1.9",
        build_file = Label("//cargo/remote:BUILD.unicode-width-0.1.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_xid__0_2_2",
        url = "https://crates.io/api/v1/crates/unicode-xid/0.2.2/download",
        type = "tar.gz",
        sha256 = "8ccb82d61f80a663efe1f787a51b16b5a51e3314d6ac365b08639f52387b33f3",
        strip_prefix = "unicode-xid-0.2.2",
        build_file = Label("//cargo/remote:BUILD.unicode-xid-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__vcpkg__0_2_15",
        url = "https://crates.io/api/v1/crates/vcpkg/0.2.15/download",
        type = "tar.gz",
        sha256 = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426",
        strip_prefix = "vcpkg-0.2.15",
        build_file = Label("//cargo/remote:BUILD.vcpkg-0.2.15.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__vec_map__0_8_2",
        url = "https://crates.io/api/v1/crates/vec_map/0.8.2/download",
        type = "tar.gz",
        sha256 = "f1bddf1187be692e79c5ffeab891132dfb0f236ed36a43c7ed39f1165ee20191",
        strip_prefix = "vec_map-0.8.2",
        build_file = Label("//cargo/remote:BUILD.vec_map-0.8.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_9_4",
        url = "https://crates.io/api/v1/crates/version_check/0.9.4/download",
        type = "tar.gz",
        sha256 = "49874b5167b65d7193b8aba1567f5c7d93d001cafc34600cee003eda787e483f",
        strip_prefix = "version_check-0.9.4",
        build_file = Label("//cargo/remote:BUILD.version_check-0.9.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__want__0_3_0",
        url = "https://crates.io/api/v1/crates/want/0.3.0/download",
        type = "tar.gz",
        sha256 = "1ce8a968cb1cd110d136ff8b819a556d6fb6d919363c61534f6860c7eb172ba0",
        strip_prefix = "want-0.3.0",
        build_file = Label("//cargo/remote:BUILD.want-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasi__0_10_2_wasi_snapshot_preview1",
        url = "https://crates.io/api/v1/crates/wasi/0.10.2+wasi-snapshot-preview1/download",
        type = "tar.gz",
        sha256 = "fd6fbd9a79829dd1ad0cc20627bf1ed606756a7f77edff7b66b7064f9cb327c6",
        strip_prefix = "wasi-0.10.2+wasi-snapshot-preview1",
        build_file = Label("//cargo/remote:BUILD.wasi-0.10.2+wasi-snapshot-preview1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__which__4_2_2",
        url = "https://crates.io/api/v1/crates/which/4.2.2/download",
        type = "tar.gz",
        sha256 = "ea187a8ef279bc014ec368c27a920da2024d2a711109bfbe3440585d5cf27ad9",
        strip_prefix = "which-4.2.2",
        build_file = Label("//cargo/remote:BUILD.which-4.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_3_9",
        url = "https://crates.io/api/v1/crates/winapi/0.3.9/download",
        type = "tar.gz",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//cargo/remote:BUILD.winapi-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_util__0_1_5",
        url = "https://crates.io/api/v1/crates/winapi-util/0.1.5/download",
        type = "tar.gz",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//cargo/remote:BUILD.winapi-util-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows__0_29_0",
        url = "https://crates.io/api/v1/crates/windows/0.29.0/download",
        type = "tar.gz",
        sha256 = "aac7fef12f4b59cd0a29339406cc9203ab44e440ddff6b3f5a41455349fa9cf3",
        strip_prefix = "windows-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows-0.29.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_aarch64_msvc__0_29_0",
        url = "https://crates.io/api/v1/crates/windows_aarch64_msvc/0.29.0/download",
        type = "tar.gz",
        sha256 = "c3d027175d00b01e0cbeb97d6ab6ebe03b12330a35786cbaca5252b1c4bf5d9b",
        strip_prefix = "windows_aarch64_msvc-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows_aarch64_msvc-0.29.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_i686_gnu__0_29_0",
        url = "https://crates.io/api/v1/crates/windows_i686_gnu/0.29.0/download",
        type = "tar.gz",
        sha256 = "8793f59f7b8e8b01eda1a652b2697d87b93097198ae85f823b969ca5b89bba58",
        strip_prefix = "windows_i686_gnu-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows_i686_gnu-0.29.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_i686_msvc__0_29_0",
        url = "https://crates.io/api/v1/crates/windows_i686_msvc/0.29.0/download",
        type = "tar.gz",
        sha256 = "8602f6c418b67024be2996c512f5f995de3ba417f4c75af68401ab8756796ae4",
        strip_prefix = "windows_i686_msvc-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows_i686_msvc-0.29.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_x86_64_gnu__0_29_0",
        url = "https://crates.io/api/v1/crates/windows_x86_64_gnu/0.29.0/download",
        type = "tar.gz",
        sha256 = "f3d615f419543e0bd7d2b3323af0d86ff19cbc4f816e6453f36a2c2ce889c354",
        strip_prefix = "windows_x86_64_gnu-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows_x86_64_gnu-0.29.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_x86_64_msvc__0_29_0",
        url = "https://crates.io/api/v1/crates/windows_x86_64_msvc/0.29.0/download",
        type = "tar.gz",
        sha256 = "11d95421d9ed3672c280884da53201a5c46b7b2765ca6faf34b0d71cf34a3561",
        strip_prefix = "windows_x86_64_msvc-0.29.0",
        build_file = Label("//cargo/remote:BUILD.windows_x86_64_msvc-0.29.0.bazel"),
    )
