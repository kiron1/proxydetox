# For Developers

## Components in this repository

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

- [pac_utils.js](paclib/src/pac_utils.js) is extracted from
  [Mozillas ProxyAutoConfig.cpp](https://dxr.mozilla.org/mozilla-central/source/netwerk/base/ProxyAutoConfig.cpp)
  and is under the MPL2 license.
