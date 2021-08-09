Proxydetox
==========

A small proxy to relive the pain of some corporate proxies.

Most utilities support the `http_proxy`, `https_proxy` and `no_proxy`
environment variables to handle proxies. On the other side some corporate
networks provide different proxies depending on the destination and for
selecting the correct proxy a [Proxy Auto-Configuration (PAC) file][mdnpac] is
provided. This is inherently incompatible with many tools who only support the
proxy environment variables. Additionally some proxies in coroprate networks
require authentication which can mean that the user name and password is stored
in plain text in environment variables.

This *Proxydetox* software is meant to help in this situation: the Proxydetox
provides a local proxy without authentication. Upon receiving a request
Proxydetox will evaluate the PAC configuration and forward to the correct
parent proxy and also optionally authenticate with them. With Proxydetox it is
enough to set a single proxy running on localhost. This should be compatible
with most tools.

The following authentication methods are supported:

- Basic: use the username and password from `~/.netrc`.
- Negotiate<sup>1</sup>: on Linux and macOS it will use [GSSAPI][gssapi], on Windows [SSPI][sspi] will be used.

Installation instructions are provided in the [INSTALL.md](./INSTALL.md) file.

<sup>1)</sup> To enable the negotiate feature, please see the instructions in
[INSTALL.md](./INSTALL.md).

[mdnpac]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_(PAC)_file "Proxy Auto-Configuration (PAC) file"
[sspi]: https://docs.microsoft.com/en-us/windows/win32/rpc/security-support-provider-interface-sspi- "Security Support Provider Interface (SSPI)"
[gssapi]: https://web.mit.edu/kerberos/krb5-devel/doc/appdev/gssapi.html "Generic Security Services API (GSSAPI)"

Alternative solutions
---------------------

- [Squid][squid]: using the[cache_peer][cache_peer] directive and translating the given PAC file into Squid ACLs.
- [SpechtLite][specht]: and translating the PAC file into the SpechtLite YAML configuration format (**unmaintained**).
- [Px][px]: A HTTP proxy server to automatically authenticate through an NTLM proxy (can handle PAC files).
- [Cntlm][cntlm]: a NTLM / NTLMv2 authenticating HTTP/1.1 proxy. Cannot handle PAC files (**unmaintained**).

[squid]: http://www.squid-cache.org "A caching proxy for the Web"
[cache_peer]: http://www.squid-cache.org/Doc/config/cache_peer/ "Squid configuration directive cache_peer"
[specht]: https://github.com/zhuhaow/SpechtLite "A rule-based proxy for macOS"
[px]: https://github.com/genotrance/px "Px"
[cntlm]: http://cntlm.sf.net/ "Cntlm Authentication Proxy"

Components in this repository
-----------------------------

- [duktape-sys](./duktape-sys/) - FFI bindings for Rust of the [duktape](https://duktape.org)
  JavaScript interpreter library written in C.
- [duktape](./duktape/) - Idiomatic Rust wrapper for the `duktape_sys` crate.
  (Just enough which is needed in this product context).
- [paclib](./paclib/) - Functions needed to implement `FindProxyForURL` and wrap it in Rust.
- [paceval](./paceval/) - A utility to evaluate PAC files for a given URL and print the result.
- [proxydetox](./proxydetox/) - The actual Proxydetox software.

Third party source code in this repository
------------------------------------------

- [duk_config.h](duktape-sys/src/duk_config.h),
  [duktape.h](duktape-sys/src/duktape.h), and
  [duktape.c](duktape-sys/src/duktape.c) are from the
  [Duktape project](https://duktape.org) and is under the MIT license.
- [pac_utils.js](paclib/src/pac_utils.js) is extracted from
  [Mozillas ProxyAutoConfig.cpp](https://dxr.mozilla.org/mozilla-central/source/netwerk/base/ProxyAutoConfig.cpp)
  and is under the MPL2 license.

License
-------

This source code is under the [MIT](https://opensource.org/licenses/MIT)
license with the exceptions mentioned in "Third party source code in
this repository".
