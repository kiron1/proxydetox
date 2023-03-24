# Rust Toolchain

```sh
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu
```

# C Toolchain

We can use [pkgsrc](https://pkgsrc.joyent.com/install-on-osx/) to
install the the mingw GNU cross compiler:

```sh
sudo pkgin install mingw-w64-x86_64-gcc mingw-w64-x86_64-winpthreads-8.0.0
```

# Cargo configuration

File `~/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
linker = "/opt/pkg/cross/x86_64-w64-mingw32/bin/x86_64-w64-mingw32-gcc"
rustc-link-search = ["/opt/pkg/cross/x86_64-w64-mingw32-winpthreads/mingw//lib/"]
```

# Cross compile

```sh
export RUSTFLAGS="-L /opt/pkg/cross/x86_64-w64-mingw32-winpthreads/mingw/lib"
cargo build --target x86_64-pc-windows-gnu
```

# Reference

- https://doc.rust-lang.org/cargo/reference/config.html
- https://doc.rust-lang.org/cargo/reference/config.html#targettriplelinker
