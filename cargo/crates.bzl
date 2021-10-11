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
        name = "raze__ansi_term__0_11_0",
        url = "https://crates.io/api/v1/crates/ansi_term/0.11.0/download",
        type = "tar.gz",
        sha256 = "ee49baf6cb617b853aa8d93bf420db2383fab46d314482ca2803b40d5fde979b",
        strip_prefix = "ansi_term-0.11.0",
        build_file = Label("//cargo/remote:BUILD.ansi_term-0.11.0.bazel"),
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
        name = "raze__argh__0_1_6",
        url = "https://crates.io/api/v1/crates/argh/0.1.6/download",
        type = "tar.gz",
        sha256 = "f023c76cd7975f9969f8e29f0e461decbdc7f51048ce43427107a3d192f1c9bf",
        strip_prefix = "argh-0.1.6",
        build_file = Label("//cargo/remote:BUILD.argh-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__argh_derive__0_1_6",
        url = "https://crates.io/api/v1/crates/argh_derive/0.1.6/download",
        type = "tar.gz",
        sha256 = "48ad219abc0c06ca788aface2e3a1970587e3413ab70acd20e54b6ec524c1f8f",
        strip_prefix = "argh_derive-0.1.6",
        build_file = Label("//cargo/remote:BUILD.argh_derive-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__argh_shared__0_1_6",
        url = "https://crates.io/api/v1/crates/argh_shared/0.1.6/download",
        type = "tar.gz",
        sha256 = "38de00daab4eac7d753e97697066238d67ce9d7e2d823ab4f72fe14af29f3f33",
        strip_prefix = "argh_shared-0.1.6",
        build_file = Label("//cargo/remote:BUILD.argh_shared-0.1.6.bazel"),
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
        name = "raze__bindgen__0_59_1",
        url = "https://crates.io/api/v1/crates/bindgen/0.59.1/download",
        type = "tar.gz",
        sha256 = "453c49e5950bb0eb63bb3df640e31618846c89d5b7faa54040d76e98e0134375",
        strip_prefix = "bindgen-0.59.1",
        build_file = Label("//cargo/remote:BUILD.bindgen-0.59.1.bazel"),
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
        name = "raze__bitvec__0_19_5",
        url = "https://crates.io/api/v1/crates/bitvec/0.19.5/download",
        type = "tar.gz",
        sha256 = "8942c8d352ae1838c9dda0b0ca2ab657696ef2232a20147cf1b30ae1a9cb4321",
        strip_prefix = "bitvec-0.19.5",
        build_file = Label("//cargo/remote:BUILD.bitvec-0.19.5.bazel"),
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
        name = "raze__cc__1_0_71",
        url = "https://crates.io/api/v1/crates/cc/1.0.71/download",
        type = "tar.gz",
        sha256 = "79c2681d6594606957bbb8631c4b90a7fcaaa72cdb714743a437b156d6a7eedd",
        strip_prefix = "cc-1.0.71",
        build_file = Label("//cargo/remote:BUILD.cc-1.0.71.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cexpr__0_5_0",
        url = "https://crates.io/api/v1/crates/cexpr/0.5.0/download",
        type = "tar.gz",
        sha256 = "db507a7679252d2276ed0dd8113c6875ec56d3089f9225b2b42c30cc1f8e5c89",
        strip_prefix = "cexpr-0.5.0",
        build_file = Label("//cargo/remote:BUILD.cexpr-0.5.0.bazel"),
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
        name = "raze__chrono__0_4_19",
        url = "https://crates.io/api/v1/crates/chrono/0.4.19/download",
        type = "tar.gz",
        sha256 = "670ad68c9088c2a963aaa298cb369688cf3f9465ce5e2d4ca10e6e0098a1ce73",
        strip_prefix = "chrono-0.4.19",
        build_file = Label("//cargo/remote:BUILD.chrono-0.4.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clang_sys__1_2_2",
        url = "https://crates.io/api/v1/crates/clang-sys/1.2.2/download",
        type = "tar.gz",
        sha256 = "10612c0ec0e0a1ff0e97980647cb058a6e7aedb913d01d009c406b8b7d0b26ee",
        strip_prefix = "clang-sys-1.2.2",
        build_file = Label("//cargo/remote:BUILD.clang-sys-1.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap__2_33_3",
        url = "https://crates.io/api/v1/crates/clap/2.33.3/download",
        type = "tar.gz",
        sha256 = "37e58ac78573c40708d45522f0d80fa2f01cc4f9b4e2bf749807255454312002",
        strip_prefix = "clap-2.33.3",
        build_file = Label("//cargo/remote:BUILD.clap-2.33.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__const_sha1__0_2_0",
        url = "https://crates.io/api/v1/crates/const-sha1/0.2.0/download",
        type = "tar.gz",
        sha256 = "fb58b6451e8c2a812ad979ed1d83378caa5e927eef2622017a45f251457c2c9d",
        strip_prefix = "const-sha1-0.2.0",
        build_file = Label("//cargo/remote:BUILD.const-sha1-0.2.0.bazel"),
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
        name = "raze__env_logger__0_8_4",
        url = "https://crates.io/api/v1/crates/env_logger/0.8.4/download",
        type = "tar.gz",
        sha256 = "a19187fea3ac7e84da7dacf48de0c45d63c6a76f9490dae389aead16c243fce3",
        strip_prefix = "env_logger-0.8.4",
        build_file = Label("//cargo/remote:BUILD.env_logger-0.8.4.bazel"),
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
        name = "raze__funty__1_1_0",
        url = "https://crates.io/api/v1/crates/funty/1.1.0/download",
        type = "tar.gz",
        sha256 = "fed34cd105917e91daa4da6b3728c47b068749d6a62c59811f06ed2ac71d9da7",
        strip_prefix = "funty-1.1.0",
        build_file = Label("//cargo/remote:BUILD.funty-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures__0_3_17",
        url = "https://crates.io/api/v1/crates/futures/0.3.17/download",
        type = "tar.gz",
        sha256 = "a12aa0eb539080d55c3f2d45a67c3b58b6b0773c1a3ca2dfec66d58c97fd66ca",
        strip_prefix = "futures-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_channel__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-channel/0.3.17/download",
        type = "tar.gz",
        sha256 = "5da6ba8c3bb3c165d3c7319fc1cc8304facf1fb8db99c5de877183c08a273888",
        strip_prefix = "futures-channel-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-channel-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_core__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-core/0.3.17/download",
        type = "tar.gz",
        sha256 = "88d1c26957f23603395cd326b0ffe64124b818f4449552f960d815cfba83a53d",
        strip_prefix = "futures-core-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-core-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_executor__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-executor/0.3.17/download",
        type = "tar.gz",
        sha256 = "45025be030969d763025784f7f355043dc6bc74093e4ecc5000ca4dc50d8745c",
        strip_prefix = "futures-executor-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-executor-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_io__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-io/0.3.17/download",
        type = "tar.gz",
        sha256 = "522de2a0fe3e380f1bc577ba0474108faf3f6b18321dbf60b3b9c39a75073377",
        strip_prefix = "futures-io-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-io-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_macro__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-macro/0.3.17/download",
        type = "tar.gz",
        sha256 = "18e4a4b95cea4b4ccbcf1c5675ca7c4ee4e9e75eb79944d07defde18068f79bb",
        strip_prefix = "futures-macro-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-macro-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_sink__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-sink/0.3.17/download",
        type = "tar.gz",
        sha256 = "36ea153c13024fe480590b3e3d4cad89a0cfacecc24577b68f86c6ced9c2bc11",
        strip_prefix = "futures-sink-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-sink-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_task__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-task/0.3.17/download",
        type = "tar.gz",
        sha256 = "1d3d00f4eddb73e498a54394f228cd55853bdf059259e8e7bc6e69d408892e99",
        strip_prefix = "futures-task-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-task-0.3.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_util__0_3_17",
        url = "https://crates.io/api/v1/crates/futures-util/0.3.17/download",
        type = "tar.gz",
        sha256 = "36568465210a3a6ee45e1f165136d68671471a501e632e9a98d96872222b5481",
        strip_prefix = "futures-util-0.3.17",
        build_file = Label("//cargo/remote:BUILD.futures-util-0.3.17.bazel"),
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
        name = "raze__h2__0_3_6",
        url = "https://crates.io/api/v1/crates/h2/0.3.6/download",
        type = "tar.gz",
        sha256 = "6c06815895acec637cd6ed6e9662c935b866d20a106f8361892893a7d9234964",
        strip_prefix = "h2-0.3.6",
        build_file = Label("//cargo/remote:BUILD.h2-0.3.6.bazel"),
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
        name = "raze__http__0_2_5",
        url = "https://crates.io/api/v1/crates/http/0.2.5/download",
        type = "tar.gz",
        sha256 = "1323096b05d41827dadeaee54c9981958c0f94e670bc94ed80037d1a7b8b186b",
        strip_prefix = "http-0.2.5",
        build_file = Label("//cargo/remote:BUILD.http-0.2.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http_body__0_4_3",
        url = "https://crates.io/api/v1/crates/http-body/0.4.3/download",
        type = "tar.gz",
        sha256 = "399c583b2979440c60be0821a6199eca73bc3c8dcd9d070d75ac726e2c6186e5",
        strip_prefix = "http-body-0.4.3",
        build_file = Label("//cargo/remote:BUILD.http-body-0.4.3.bazel"),
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
        name = "raze__httpdate__1_0_1",
        url = "https://crates.io/api/v1/crates/httpdate/1.0.1/download",
        type = "tar.gz",
        sha256 = "6456b8a6c8f33fee7d958fcd1b60d55b11940a79e63ae87013e6d22e26034440",
        strip_prefix = "httpdate-1.0.1",
        build_file = Label("//cargo/remote:BUILD.httpdate-1.0.1.bazel"),
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
        name = "raze__hyper__0_14_13",
        url = "https://crates.io/api/v1/crates/hyper/0.14.13/download",
        type = "tar.gz",
        sha256 = "15d1cfb9e4f68655fa04c01f59edb405b6074a0f7118ea881e5026e4a1cd8593",
        strip_prefix = "hyper-0.14.13",
        build_file = Label("//cargo/remote:BUILD.hyper-0.14.13.bazel"),
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
        name = "raze__inotify__0_9_5",
        url = "https://crates.io/api/v1/crates/inotify/0.9.5/download",
        type = "tar.gz",
        sha256 = "9e5fc8f41dbaa9c8492a96c8afffda4f76896ee041d6a57606e70581b80c901f",
        strip_prefix = "inotify-0.9.5",
        build_file = Label("//cargo/remote:BUILD.inotify-0.9.5.bazel"),
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
        name = "raze__libc__0_2_103",
        url = "https://crates.io/api/v1/crates/libc/0.2.103/download",
        type = "tar.gz",
        sha256 = "dd8f7255a17a627354f321ef0055d63b898c6fb27eff628af4d1b66b7331edf6",
        strip_prefix = "libc-0.2.103",
        build_file = Label("//cargo/remote:BUILD.libc-0.2.103.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libloading__0_7_1",
        url = "https://crates.io/api/v1/crates/libloading/0.7.1/download",
        type = "tar.gz",
        sha256 = "c0cf036d15402bea3c5d4de17b3fce76b3e4a56ebc1f577be0e7a72f7c607cf0",
        strip_prefix = "libloading-0.7.1",
        build_file = Label("//cargo/remote:BUILD.libloading-0.7.1.bazel"),
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
        name = "raze__matchers__0_0_1",
        url = "https://crates.io/api/v1/crates/matchers/0.0.1/download",
        type = "tar.gz",
        sha256 = "f099785f7595cc4b4553a174ce30dd7589ef93391ff414dbb67f62392b9e0ce1",
        strip_prefix = "matchers-0.0.1",
        build_file = Label("//cargo/remote:BUILD.matchers-0.0.1.bazel"),
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
        name = "raze__mio__0_7_13",
        url = "https://crates.io/api/v1/crates/mio/0.7.13/download",
        type = "tar.gz",
        sha256 = "8c2bdb6314ec10835cd3293dd268473a835c02b7b352e788be788b3c6ca6bb16",
        strip_prefix = "mio-0.7.13",
        build_file = Label("//cargo/remote:BUILD.mio-0.7.13.bazel"),
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
        new_git_repository,
        name = "raze__netrc__0_4_1",
        remote = "https://github.com/kiron1/netrc-rs",
        commit = "82aa95f8d43b480f9ddb1695f0e4d73d6e168cb5",
        build_file = Label("//cargo/remote:BUILD.netrc-0.4.1.bazel"),
        init_submodules = True,
    )

    maybe(
        http_archive,
        name = "raze__nom__6_1_2",
        url = "https://crates.io/api/v1/crates/nom/6.1.2/download",
        type = "tar.gz",
        sha256 = "e7413f999671bd4745a7b624bd370a569fb6bc574b23c83a3c5ed2e453f3d5e2",
        strip_prefix = "nom-6.1.2",
        build_file = Label("//cargo/remote:BUILD.nom-6.1.2.bazel"),
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
        name = "raze__num_integer__0_1_44",
        url = "https://crates.io/api/v1/crates/num-integer/0.1.44/download",
        type = "tar.gz",
        sha256 = "d2cc698a63b549a70bc047073d2949cce27cd1c7b0a4a862d08a8031bc2801db",
        strip_prefix = "num-integer-0.1.44",
        build_file = Label("//cargo/remote:BUILD.num-integer-0.1.44.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_2_14",
        url = "https://crates.io/api/v1/crates/num-traits/0.2.14/download",
        type = "tar.gz",
        sha256 = "9a64b1ec5cda2586e284722486d802acf1f7dbdc623e2bfc57e65ca1cd099290",
        strip_prefix = "num-traits-0.2.14",
        build_file = Label("//cargo/remote:BUILD.num-traits-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_cpus__1_13_0",
        url = "https://crates.io/api/v1/crates/num_cpus/1.13.0/download",
        type = "tar.gz",
        sha256 = "05499f3756671c15885fee9034446956fff3f243d6077b91e5767df161f766b3",
        strip_prefix = "num_cpus-1.13.0",
        build_file = Label("//cargo/remote:BUILD.num_cpus-1.13.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__once_cell__1_8_0",
        url = "https://crates.io/api/v1/crates/once_cell/1.8.0/download",
        type = "tar.gz",
        sha256 = "692fcb63b64b1758029e0a96ee63e049ce8c5948587f2f7208df04625e5f6b56",
        strip_prefix = "once_cell-1.8.0",
        build_file = Label("//cargo/remote:BUILD.once_cell-1.8.0.bazel"),
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
        name = "raze__pin_project__1_0_8",
        url = "https://crates.io/api/v1/crates/pin-project/1.0.8/download",
        type = "tar.gz",
        sha256 = "576bc800220cc65dac09e99e97b08b358cfab6e17078de8dc5fee223bd2d0c08",
        strip_prefix = "pin-project-1.0.8",
        build_file = Label("//cargo/remote:BUILD.pin-project-1.0.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_project_internal__1_0_8",
        url = "https://crates.io/api/v1/crates/pin-project-internal/1.0.8/download",
        type = "tar.gz",
        sha256 = "6e8fe8163d14ce7f0cdac2e040116f22eac817edabff0be91e8aff7e9accf389",
        strip_prefix = "pin-project-internal-1.0.8",
        build_file = Label("//cargo/remote:BUILD.pin-project-internal-1.0.8.bazel"),
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
        name = "raze__proc_macro_hack__0_5_19",
        url = "https://crates.io/api/v1/crates/proc-macro-hack/0.5.19/download",
        type = "tar.gz",
        sha256 = "dbf0c48bc1d91375ae5c3cd81e3722dff1abcf81a30960240640d223f59fe0e5",
        strip_prefix = "proc-macro-hack-0.5.19",
        build_file = Label("//cargo/remote:BUILD.proc-macro-hack-0.5.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro_nested__0_1_7",
        url = "https://crates.io/api/v1/crates/proc-macro-nested/0.1.7/download",
        type = "tar.gz",
        sha256 = "bc881b2c22681370c6a780e47af9840ef841837bc98118431d4e1868bd0c1086",
        strip_prefix = "proc-macro-nested-0.1.7",
        build_file = Label("//cargo/remote:BUILD.proc-macro-nested-0.1.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_29",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.29/download",
        type = "tar.gz",
        sha256 = "b9f5105d4fdaab20335ca9565e106a5d9b82b6219b5ba735731124ac6711d23d",
        strip_prefix = "proc-macro2-1.0.29",
        build_file = Label("//cargo/remote:BUILD.proc-macro2-1.0.29.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_10",
        url = "https://crates.io/api/v1/crates/quote/1.0.10/download",
        type = "tar.gz",
        sha256 = "38bc8cc6a5f2e3655e0899c1b848643b2562f853f114bfec7be120678e3ace05",
        strip_prefix = "quote-1.0.10",
        build_file = Label("//cargo/remote:BUILD.quote-1.0.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__radium__0_5_3",
        url = "https://crates.io/api/v1/crates/radium/0.5.3/download",
        type = "tar.gz",
        sha256 = "941ba9d78d8e2f7ce474c015eea4d9c6d25b6a3327f9832ee29a4de27f91bbb8",
        strip_prefix = "radium-0.5.3",
        build_file = Label("//cargo/remote:BUILD.radium-0.5.3.bazel"),
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
        name = "raze__regex_automata__0_1_10",
        url = "https://crates.io/api/v1/crates/regex-automata/0.1.10/download",
        type = "tar.gz",
        sha256 = "6c230d73fb8d8c1b9c0b3135c5142a8acee3a0558fb8db5cf1cb65f8d7862132",
        strip_prefix = "regex-automata-0.1.10",
        build_file = Label("//cargo/remote:BUILD.regex-automata-0.1.10.bazel"),
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
        name = "raze__rustc_hash__1_1_0",
        url = "https://crates.io/api/v1/crates/rustc-hash/1.1.0/download",
        type = "tar.gz",
        sha256 = "08d43f7aa6b08d49f382cde6a7982047c3426db949b1424bc4b7ec9ae12c6ce2",
        strip_prefix = "rustc-hash-1.1.0",
        build_file = Label("//cargo/remote:BUILD.rustc-hash-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ryu__1_0_5",
        url = "https://crates.io/api/v1/crates/ryu/1.0.5/download",
        type = "tar.gz",
        sha256 = "71d301d4193d031abdd79ff7e3dd721168a9572ef3fe51a1517aba235bd8f86e",
        strip_prefix = "ryu-1.0.5",
        build_file = Label("//cargo/remote:BUILD.ryu-1.0.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__1_0_130",
        url = "https://crates.io/api/v1/crates/serde/1.0.130/download",
        type = "tar.gz",
        sha256 = "f12d06de37cf59146fbdecab66aa99f9fe4f78722e3607577a5375d66bd0c913",
        strip_prefix = "serde-1.0.130",
        build_file = Label("//cargo/remote:BUILD.serde-1.0.130.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_json__1_0_68",
        url = "https://crates.io/api/v1/crates/serde_json/1.0.68/download",
        type = "tar.gz",
        sha256 = "0f690853975602e1bfe1ccbf50504d67174e3bcf340f23b5ea9992e0587a52d8",
        strip_prefix = "serde_json-1.0.68",
        build_file = Label("//cargo/remote:BUILD.serde_json-1.0.68.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sharded_slab__0_1_3",
        url = "https://crates.io/api/v1/crates/sharded-slab/0.1.3/download",
        type = "tar.gz",
        sha256 = "740223c51853f3145fe7c90360d2d4232f2b62e3449489c207eccde818979982",
        strip_prefix = "sharded-slab-0.1.3",
        build_file = Label("//cargo/remote:BUILD.sharded-slab-0.1.3.bazel"),
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
        name = "raze__slab__0_4_4",
        url = "https://crates.io/api/v1/crates/slab/0.4.4/download",
        type = "tar.gz",
        sha256 = "c307a32c1c5c437f38c7fd45d753050587732ba8628319fbdf12a7e289ccc590",
        strip_prefix = "slab-0.4.4",
        build_file = Label("//cargo/remote:BUILD.slab-0.4.4.bazel"),
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
        name = "raze__syn__1_0_80",
        url = "https://crates.io/api/v1/crates/syn/1.0.80/download",
        type = "tar.gz",
        sha256 = "d010a1623fbd906d51d650a9916aaefc05ffa0e4053ff7fe601167f3e715d194",
        strip_prefix = "syn-1.0.80",
        build_file = Label("//cargo/remote:BUILD.syn-1.0.80.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tap__1_0_1",
        url = "https://crates.io/api/v1/crates/tap/1.0.1/download",
        type = "tar.gz",
        sha256 = "55937e1799185b12863d447f42597ed69d9928686b8d88a1df17376a097d8369",
        strip_prefix = "tap-1.0.1",
        build_file = Label("//cargo/remote:BUILD.tap-1.0.1.bazel"),
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
        name = "raze__tokio__1_12_0",
        url = "https://crates.io/api/v1/crates/tokio/1.12.0/download",
        type = "tar.gz",
        sha256 = "c2c2416fdedca8443ae44b4527de1ea633af61d8f7169ffa6e72c5b53d24efcc",
        strip_prefix = "tokio-1.12.0",
        build_file = Label("//cargo/remote:BUILD.tokio-1.12.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_macros__1_4_1",
        url = "https://crates.io/api/v1/crates/tokio-macros/1.4.1/download",
        type = "tar.gz",
        sha256 = "154794c8f499c2619acd19e839294703e9e32e7630ef5f46ea80d4ef0fbee5eb",
        strip_prefix = "tokio-macros-1.4.1",
        build_file = Label("//cargo/remote:BUILD.tokio-macros-1.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_util__0_6_8",
        url = "https://crates.io/api/v1/crates/tokio-util/0.6.8/download",
        type = "tar.gz",
        sha256 = "08d3725d3efa29485e87311c5b699de63cde14b00ed4d256b8318aa30ca452cd",
        strip_prefix = "tokio-util-0.6.8",
        build_file = Label("//cargo/remote:BUILD.tokio-util-0.6.8.bazel"),
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
        name = "raze__tracing_serde__0_1_2",
        url = "https://crates.io/api/v1/crates/tracing-serde/0.1.2/download",
        type = "tar.gz",
        sha256 = "fb65ea441fbb84f9f6748fd496cf7f63ec9af5bca94dd86456978d055e8eb28b",
        strip_prefix = "tracing-serde-0.1.2",
        build_file = Label("//cargo/remote:BUILD.tracing-serde-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_subscriber__0_2_25",
        url = "https://crates.io/api/v1/crates/tracing-subscriber/0.2.25/download",
        type = "tar.gz",
        sha256 = "0e0d2eaa99c3c2e41547cfa109e910a68ea03823cccad4a0525dcbc9b01e8c71",
        strip_prefix = "tracing-subscriber-0.2.25",
        build_file = Label("//cargo/remote:BUILD.tracing-subscriber-0.2.25.bazel"),
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
        name = "raze__vec_map__0_8_2",
        url = "https://crates.io/api/v1/crates/vec_map/0.8.2/download",
        type = "tar.gz",
        sha256 = "f1bddf1187be692e79c5ffeab891132dfb0f236ed36a43c7ed39f1165ee20191",
        strip_prefix = "vec_map-0.8.2",
        build_file = Label("//cargo/remote:BUILD.vec_map-0.8.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_9_3",
        url = "https://crates.io/api/v1/crates/version_check/0.9.3/download",
        type = "tar.gz",
        sha256 = "5fecdca9a5291cc2b8dcf7dc02453fee791a280f3743cb0905f8822ae463b3fe",
        strip_prefix = "version_check-0.9.3",
        build_file = Label("//cargo/remote:BUILD.version_check-0.9.3.bazel"),
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
        name = "raze__which__3_1_1",
        url = "https://crates.io/api/v1/crates/which/3.1.1/download",
        type = "tar.gz",
        sha256 = "d011071ae14a2f6671d0b74080ae0cd8ebf3a6f8c9589a2cd45f23126fe29724",
        strip_prefix = "which-3.1.1",
        build_file = Label("//cargo/remote:BUILD.which-3.1.1.bazel"),
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
        name = "raze__windows__0_21_1",
        url = "https://crates.io/api/v1/crates/windows/0.21.1/download",
        type = "tar.gz",
        sha256 = "b8f5f8d2ea79bf690bbee453fd4a1516ae426e5d5c7215d96cc0c3dc134fc4a0",
        strip_prefix = "windows-0.21.1",
        build_file = Label("//cargo/remote:BUILD.windows-0.21.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_gen__0_21_1",
        url = "https://crates.io/api/v1/crates/windows_gen/0.21.1/download",
        type = "tar.gz",
        sha256 = "7e6994f42f8481387778cc608407d6703410672d57f32a66009419d7a18aa912",
        strip_prefix = "windows_gen-0.21.1",
        build_file = Label("//cargo/remote:BUILD.windows_gen-0.21.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_macros__0_21_1",
        url = "https://crates.io/api/v1/crates/windows_macros/0.21.1/download",
        type = "tar.gz",
        sha256 = "81cc2357b1b03c19f056cb0e6d06011f80f54beadb4e36aee2ca98493c7cfc3c",
        strip_prefix = "windows_macros-0.21.1",
        build_file = Label("//cargo/remote:BUILD.windows_macros-0.21.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_quote__0_21_1",
        url = "https://crates.io/api/v1/crates/windows_quote/0.21.1/download",
        type = "tar.gz",
        sha256 = "7cf987b5288c15e1997226848f78f3ed3ef8b78dcfd71a201c8c8684163a7e4d",
        strip_prefix = "windows_quote-0.21.1",
        build_file = Label("//cargo/remote:BUILD.windows_quote-0.21.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_reader__0_21_1",
        url = "https://crates.io/api/v1/crates/windows_reader/0.21.1/download",
        type = "tar.gz",
        sha256 = "237b53e8b40766ea7db5da0d8c6c1442d21d0429f0ee7500d7b5688967bd9d7b",
        strip_prefix = "windows_reader-0.21.1",
        build_file = Label("//cargo/remote:BUILD.windows_reader-0.21.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wyz__0_2_0",
        url = "https://crates.io/api/v1/crates/wyz/0.2.0/download",
        type = "tar.gz",
        sha256 = "85e60b0d1b5f99db2556934e21937020776a5d31520bf169e851ac44e6420214",
        strip_prefix = "wyz-0.2.0",
        build_file = Label("//cargo/remote:BUILD.wyz-0.2.0.bazel"),
    )
