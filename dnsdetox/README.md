# DNS detox

A small DNS proxy to relief the pain of some cooperate networks (CN).

In some cooperate networks, the internal DNS server only response with answers
for IP addresses which lay inside the intranet. Resources from the internet
cannot be resolved. This DNS proxy will first try to resolve via the intranet
DNS server and when this fails it will use DNS over HTTPS (DoH) to resolve the
IP address or an internet resource.
