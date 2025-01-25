---
name: Bug report
about: Create a report to help us improve
title: ''
labels: ''
assignees: kiron1
---

**Describe the bug**

A clear and concise description of what the bug is.

**Expected behavior**

A clear and concise description of what you expected to happen.

**PAC file content**

Content of PAC file to reproduce this issue:

```js
function FindProxyForURL(url, host) {
  return "...";
}
```

For information about Proxy Auto-Configuration (PAC) file, see:
https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_PAC_file

**Configuration**

Content of `proxydetoxrc` file or any other flags used:

**Log output**

Run `proxydetox -vv` (verbose) to get the log output when reproducing the issue.

**Operating System**

- OS: [e.g. iOS]
- Version [e.g. 22]

**Proxydetox version**

Output of `proxydetox --version`:

**Additional context**

Add any other context about the problem here.
