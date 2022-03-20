# Usage

With Proxydetox running, it is enough for most tools to set the `http_proxy` and
`https_proxy` environment variables.

```sh
http_proxy=http://127.0.0.1:3128
https_proxy=http://127.0.0.1:3128
no_proxy=127.0.0.1,localhost,::1
export http_proxy https_proxy no_proxy
```

To make this changes persistent add them to `~/.profile`, `~/.bashrc`, or
`~/.zshrc` depending on which shell you are using.
