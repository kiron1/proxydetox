# For Developers

## Components in this repository

### JavaScript support

Since the PAC files are actual JavaScript code, Proxydetox needs to be able to
run JavaScript code.

- [duktape-sys](./duktape-sys/) - FFI bindings for Rust of the
  [duktape](https://duktape.org) JavaScript interpreter library written in C.
- [duktape](./duktape/) - Idiomatic Rust wrapper for the `duktape_sys` crate.
  (Just enough which is needed in this product context).

### Evaluation of PAC scripts

- [paclib](./paclib/) - Functions needed to implement `FindProxyForURL` and wrap
  it in Rust.
- [paceval](./paceval/) - A utility to evaluate PAC files for a given URL and
  print the result.

### HTTP Proxy code

- [proxy_client](./proxy_client/) - Supporting code to allow [`hyper`][hyper] to
  utilize HTTP proxies.
- [proxydetox](./proxydetox/) - The actual Proxydetox software.

[windows-rs]: https://github.com/microsoft/windows-rs "Rust for the Windows SDK"
[hyper]: https://github.com/hyperium/hyper "A fast and correct HTTP implementation for Rust"

## Third party source code in this repository

- [duk_config.h](duktape-sys/src/duk_config.h),
  [duktape.h](duktape-sys/src/duktape.h), and
  [duktape.c](duktape-sys/src/duktape.c) are from the
  [Duktape project](https://duktape.org) and is under the MIT license.
- [pac_utils.js](paclib/src/pac_utils.js) is extracted from
  [Mozillas ProxyAutoConfig.cpp](https://dxr.mozilla.org/mozilla-central/source/netwerk/base/ProxyAutoConfig.cpp)
  and is under the MPL2 license.
