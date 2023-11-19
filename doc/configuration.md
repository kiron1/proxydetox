# Configuration

## File locations

The following files are read during startup by Proxydetox. These files can be
used to tweak the behaviour of Proxydetox.

- `proxydetoxrc`: Main configuration file.
- `proxy.pac`: The PAC file, which defines the rules to select the correct
  upstream proxy.
- `.netrc`: Contains the authentication information when `--negotiate` is _not_
  used.

The configuration files are searched at the following locations:

The `.netrc` file is expected to be located in the `HOME` directory on all
platforms. If needed, the location can be specified via the `--netrc-file` flag
when invoking Proxydetox.

The configuration files `proxydetoxrc` and `proxy.pac` are searched at user
configurable location and system wide locations.

For the different platforms, the user configurable location is as follows:

- `~/.config/proxydetox/` (POSIX-compliant systems like **BSD** and **Linux**)
- `~/Library/Application\ Support/proxydetox` (**macOS**)
- `%APPDATA%` (**Windows**)

For the different platforms, the user system wide location is as follows:

POSIX-compliant systems like **BSD**, **macOS**, and **Linux**:

- `/usr/etc/proxydetox`
- `/usr/local/etc/proxydetox/`
- `/opt/proxydetox/etc/`

Windows:

- `.` (the directory of the executable)

## `proxydetoxrc` file format

The `proxydetoxrc` file lists all options which are usually provided via the
command line to `proxydetox`.

### Example

```sh
proxydeox --negotiate --port 8080 --pac-file http://example.org/proxy.pac
```

Is equivalent with a `proxydetoxrc` file at one of the well known locations
listed above with the following content:

```
--negotiate
--port 8080
--pac-file http://example.org/proxy.pac
```

## Negotiate authentication

To enable the Negotiate authentication the `--negotiate` flag must be added when
calling `proxydetox` or added to the `proxydetoxrc` configuration file.

## Basic authentication

When the basic authentication shall be used (default), the credentials are read
from the `~/.netrc` file. An example `~/.netrc` file will look as follows:

```
machine proxy.example.org
login ProxyUsername
password ProxyPassword
```

The basic authentication is insecure, since it required to store the password in
clear text on disk and the password is transferred unencrypted.

## Proxy Auto-Configuration (PAC) file

A copy of the PAC file `proxy.pac` must be places in on of directories searched
by Proxydetox or specified via the `--pac-file` option. The PAC file can be also
a HTTP URL. The content will then be downloaded from the given location.

The PAC file is usually
maintained by the network administrators. The path (usually some http location
in the intranet) can be retrieved from the settings of the pre-configured
internet browser.

### Examples

```
proxydetox --pac-file http://example.org/proxy.pac
```

```
proxydetox --pac-file /tmp/test.pac
```

## Configuration options

The full list of configuration options can be retrieved with:

```sh
proxydetox --help
```
