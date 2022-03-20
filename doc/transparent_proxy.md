```sh
sysctl -w net.ipv4.ip_forward=1
```

```sh
addgroup --system proxydetox
usermod -aG proxydetox $(id -u)
```

```sh
iptables -A OUTPUT -t nat -p tcp --dport 80 -m owner ! --gid-owner proxydetox -j DNAT --to 127.0.0.1:3125
```

```sh
iptables -t nat -L -v --line-numbers
```

Run `proxydetox` in the `proxydetox` group sucht that the own `proxydetox`
traffic does not get matched with the `iptables` rule from above (otherwise we
would end up in a endless loop).

```sh
sg proxydetox -c proxydetox
```
