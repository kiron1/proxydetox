# Building Proxydetox

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

The easiest way to obtain Bazel, is by using Bazelisk. The remaining document
assumes that either Bazel is installed and available via the `PATH` variable or
Bazelisk is installed and the binary is named `bazel` and available via the
`PATH` variable.

For the Bazel setup to work, ensue the XCode command line tools are installed:

```sh
xcode-select --install
```

### Building ProxydetoxApp (macOS UI)

```sh
bazel build //macos/app:ProxydetoxApp
```

[bazelisk]: https://github.com/bazelbuild/bazelisk/releases "A user-friendly launcher for Bazel"

### Enable build features

Append `--features=negotiate` to `bazel build` to enable the negotiate feature:

```sh
bazel build --features=negotiate //...
```

## Autostart

To start Proxydetox automatically when a user is logged in, please see the
[Autostart](service.md) section.
