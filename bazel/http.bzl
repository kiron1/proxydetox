load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def versioned_http_archive(name, version, **kwargs):
    """Like http_archive, but replaces `{version}` in the `strip_prefix, `url`, and `urls` argument.

    Args:
        name: Name of remote repository.
        version: Version to use in replacement.
        **kwargs: See http_archive
    """
    for k in ["url", "strip_prefix"]:
        if k in kwargs:
            kwargs[k] = kwargs[k].format(version = version)
    if "urls" in kwargs:
        kwargs["urls"] = [u.format(version = version) for u in kwargs["urls"]]
    http_archive(name = name, **kwargs)
