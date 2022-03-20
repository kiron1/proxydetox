# Alternative solutions

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
