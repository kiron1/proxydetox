# Proxydetox

![Proxydetox build status of main branch](https://github.com/kiron1/proxydetox/actions/workflows/main.yaml/badge.svg)

A small proxy to relieve the pain of some corporate proxies.

Proxydetox can act as an intermediate HTTP proxy for your local applications
and actual HTTP proxy. Proxydetox will select the correct upstream proxy based
on the [Proxy Auto-Configuration (PAC) file][mdnpac] provided by the network
administrator and will take care to correctly authenticate against the upstream
proxy. The [Basic][basic] and [Negotiate][negotiate] authentication methods are
supported.

With Proxydetox in place, most local applications can be configured to use the
proxy by setting the environment variables `http_proxy`, and `https_proxy`.

Proxydetox is compatible with POSIX-compliant systems (tested on Ubuntu and
OpenBSD), macOS, and Windows.

**Get started** by looking over the [documentation](https://proxydetox.colorto.cc/).

[mdnpac]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_(PAC)_file "Proxy Auto-Configuration (PAC) file"
[basic]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication#basic_authentication_scheme "Basic authentication scheme"
[negotiate]: https://www.rfc-editor.org/rfc/rfc4559.html#section-4 "HTTP Negotiate Authentication Scheme"
[releases]: https://github.com/kiron1/proxydetox/releases "Proxydetox releases"

## License

This source code is under the [MIT](https://opensource.org/licenses/MIT) license
with the exceptions mentioned in "Third party source code in this repository".
