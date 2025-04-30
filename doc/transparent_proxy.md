# Transparent mode

The transparent mode describes an operation mode currently only available on Linux where applications without any knowledge of any Proxy can use Proxydetox.
Applications do not need to be aware of the proxy nore do they need to be configured in any way.
This approach works only for TCP based connections i.e., it cannot be used for UDP based protocols.

For the transparent mode `iptables` or `nftables` are used to redirect outgoing traffic via Proxydetox. Proxydetox contains a special handling for this kind of connections and forwards it to the correct upstream proxy.

For this to work the PAC file must handle IP based rules correctly (since the destination hostname is not available anymore since the hostname got resolved before the connection reaches Proxydeotx).

```sh
sysctl -w net.ipv4.ip_forward=1
```

```sh
iptables -A OUTPUT -t nat -p tcp --dport 80 -j DNAT --to 127.0.0.1:3128
```

```sh
iptables -t nat -L -v --line-numbers
```

With this approach it must be ensured that the firewall rules are defined such, that the Proxydetox outgoing traffic is not redirect again to Proxydetox in an infinite loop.
