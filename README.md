# Proxydetox

![Proxydetox build status of main branch](https://github.com/kiron1/proxydetox/actions/workflows/main.yaml/badge.svg)

A small proxy to relive the pain of some corporate proxies.

Proxydetox can act as an intermediate HTTP proxy for your local applications and
actual HTTP proxy. Proxydtox will talk to the actual corporate HTTP proxies on
behalf of the actual application. Proxydetox will select the correct upstream
proxy based on the [Proxy Auto-Configuration (PAC) file][mdnpac] provided by the
network administrator and will take care to correctly authenticate against the
upstream proxy.

With Proxydetox in place, most local applications can be configured to use the
proxy by setting the environment variables `http_proxy`, and `https_proxy`.

The following authentication methods are supported:

- [Basic][basic]: use the username and password from `~/.netrc`.
- [Negotiate][negotiate]<sup>1,2</sup>: on Linux and macOS it will use
  [GSSAPI][gssapi], on Windows [SSPI][sspi] will be used.

Proxydetox supports the following systems:

- POSIX-compliant systems (tested on Ubuntu and OpenBSD)
- macOS
- Windows

Prebuild versions of Proxydetox can be found on [the releaes page][releases].
Installation instructions are provided in the [INSTALL.md](./INSTALL.md) file.

<sup>1)</sup> Can be disabled via the `--no-default-features` flag during build
time.<br>
<sup>2)</sup> Can be activated with the `--negotiate` flag during runtime.

[mdnpac]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_(PAC)_file "Proxy Auto-Configuration (PAC) file"
[basic]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication#basic_authentication_scheme "Basic authentication scheme"
[negotiate]: https://www.rfc-editor.org/rfc/rfc4559.html#section-4 "HTTP Negotiate Authentication Scheme"
[sspi]: https://docs.microsoft.com/en-us/windows/win32/rpc/security-support-provider-interface-sspi- "Security Support Provider Interface (SSPI)"
[gssapi]: https://web.mit.edu/kerberos/krb5-devel/doc/appdev/gssapi.html "Generic Security Services API (GSSAPI)"
[releases]: https://github.com/kiron1/proxydetox/releases "Proxydetox releases"

## Alternative solutions

- [Squid][squid]: using the[cache_peer][cache_peer] directive and translating
  the given PAC file into Squid ACLs.
- [SpechtLite][specht]: and translating the PAC file into the SpechtLite YAML
  configuration format (**unmaintained**).
- [Px][px]: A HTTP proxy server to automatically authenticate through an NTLM
  proxy (can handle PAC files).
- [Cntlm][cntlm]: a NTLM / NTLMv2 authenticating HTTP/1.1 proxy. Cannot handle
  PAC files (**unmaintained**).

[squid]: http://www.squid-cache.org "A caching proxy for the Web"
[cache_peer]: http://www.squid-cache.org/Doc/config/cache_peer/ "Squid configuration directive cache_peer"
[specht]: https://github.com/zhuhaow/SpechtLite "A rule-based proxy for macOS"
[px]: https://github.com/genotrance/px "Px"
[cntlm]: http://cntlm.sf.net/ "Cntlm Authentication Proxy"

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

## License

This source code is under the [MIT](https://opensource.org/licenses/MIT) license
with the exceptions mentioned in "Third party source code in this repository".
