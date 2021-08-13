# Build and Install `proxydetox`

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

## Enable build features

To enable the Negotiate authentication method, the `negotiate` feature must be
enabled. This means, we would need to add `--features negotiate` to the above
`cargo install ...` command.

On GNU/Linux and macOS the
[Generic Security Services Application Program Interface (GSSAPI)][gssapi] will be used.
On Windows the [Security Support Provider Interface][sspi] is used .

[gssapi]: https://en.wikipedia.org/wiki/Generic_Security_Services_Application_Program_Interface
[sspi]: https://en.wikipedia.org/wiki/Security_Support_Provider_Interface
