# Build and Install `proxydetox`

Proxydetox can be build using [cargo][cargo] or [Bazel][bazel]. The macOS
application can only build via Bazel.

- [Build using `cargo`](#using-cargo)
- [Build using `bazel`](#using-bazel)

[cargo]: https://doc.rust-lang.org/cargo/ "Cargo is the Rust package manager"
[bazel]: https://bazel.build "Build and test software of any size, quickly and reliably"

## Using cargo

The easiest is, to use `cargo` from [rustup][rustup]. The next command will
install the `proxydetox` binary in `~/.cargo/bin`.

```sh
cargo install --git https://github.com/kiron1/proxydetox.git
```

If you have cloned this repository already, you can also do:

```sh
cargo install --path .
```

[rustup]: https://rustup.rs/ "rustup.rs - The Rust toolchain installer"

### Enable build features

To enable the Negotiate authentication method, the `negotiate` feature must be
enabled. This means, we would need to add `--features negotiate` to the above
`cargo install ...` command.

On GNU/Linux and macOS the
[Generic Security Services Application Program Interface (GSSAPI)][gssapi] will
be used. On Windows the [Security Support Provider Interface][sspi] is used .

[gssapi]: https://en.wikipedia.org/wiki/Generic_Security_Services_Application_Program_Interface
[sspi]: https://en.wikipedia.org/wiki/Security_Support_Provider_Interface

## Using Bazel

The easiest way to optiain Bazel, is by using Bazelisk. The remaining document
assumes that eiter Bazel is installed and available via the `PATH` variable or
Bazelisk is installed and the binary is named `bazel` and available via the
`PATH` variable.

Currently the Bazel setup has some quirks which requires some extra steps:

1. Ensure the XCode command line tools are installed:

```sh
xcode-select --install
```

2. Ensure clang_sys can find `clang`, by placing the following content in a file
   called `user.bazelrc` in the root directroy of our Proxydetox checkout:

Until
[KyleMayes/clang-sys/ #132](https://github.com/KyleMayes/clang-sys/pull/132) is
resolved:

```
build --action_env=PATH=/bin:/usr/bin:/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:~/.cargo/bin
```

3. Ensure the `duktape-sys` can find `TargetConditionals.h`

Currently we hardcode the necessary include path in
[`duktape-sys/BUILD`](./duktape-sys/BUILD) this might need adjustment for your
system. See
[bazelbuild/rules_rust #899](https://github.com/bazelbuild/rules_rust/issues/899)

### Buliding ProxydetoxApp (macOS UI)

```sh
bazel build //macos/app:ProxydetoxApp
```

[bazelisk]: https://github.com/bazelbuild/bazelisk/releases "A user-friendly launcher for Bazel"
