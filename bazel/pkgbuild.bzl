"""
Provide a rule to run `pkgbuild` via Bazel.
"""

load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo", "PackageVariablesInfo")

def _substitute_package_variables(ctx, attribute_value):
    # From: https://github.com/bazelbuild/rules_pkg/blob/main/pkg/private/util.bzl#L66
    vars = dict(ctx.var)
    if ctx.attr.package_variables:
        package_variables = ctx.attr.package_variables[PackageVariablesInfo]
        vars.update(package_variables.values)
    return attribute_value.replace("$(", "{").replace(")", "}").format(**vars)

def _setup_output_files(ctx):
    default_output = ctx.outputs.out

    outputs = [default_output]
    package_file_name = ctx.attr.package_file_name
    if package_file_name:
        output_name = _substitute_package_variables(ctx, package_file_name)
        output_file = ctx.actions.declare_file(output_name)
        outputs.append(output_file)
        ctx.actions.symlink(
            output = default_output,
            target_file = output_file,
        )
    else:
        output_file = default_output
        output_name = output_file.basename
    return outputs, output_file, output_name

def _pkgbuild_impl(ctx):
    """Run `pkgbuild` as a Bazel action.

    :param ctx: Bazel rule context.
    """

    outputs, output_file, output_name = _setup_output_files(
        ctx,
    )

    inputs = [ctx.file.data]
    args = [
        "--out",
        output_file.path,
        "--data",
        ctx.file.data.path,
        "--install-location",
        ctx.attr.install_location,
        "--identifier",
        ctx.attr.identifier,
        "--version",
        ctx.attr.version,
    ]

    if ctx.attr.version_file:
        if ctx.attr.version:
            fail("Both version and version_file attributes were specified")
        args.append("--version")
        args.append("@{}".format(ctx.file.version_file.path))
        inputs.append(ctx.file.version_file)
    elif ctx.attr.version:
        args.append("--version")
        args.append(ctx.file.version_file.path)
    else:
        fail("Neither version_file nor version attribute was specified")

    if ctx.attr.preinstall:
        args.append("--preinstall")
        args.append(ctx.file.preinstall.path)
        inputs.append(ctx.file.preinstall)
    if ctx.attr.postinstall:
        args.append("--postinstall")
        args.append(ctx.file.postinstall.path)
        inputs.append(ctx.file.postinstall)

    # out = ctx.actions.declare_file(ctx.outputs.out)
    ctx.actions.run(
        inputs = inputs,
        outputs = [output_file],
        arguments = args,
        executable = ctx.executable._pkgbuild,
        execution_requirements = {
            "local": "1",
            "no-remote": "1",
            "no-remote-exec": "1",
        },
    )

    output_groups = {
        "out": [ctx.outputs.out],
        "pkg": [output_file],
    }
    return [
        OutputGroupInfo(**output_groups),
        DefaultInfo(
            files = depset([output_file]),
            runfiles = ctx.runfiles(files = outputs),
        ),
        PackageArtifactInfo(
            label = ctx.label.name,
            file = output_file,
            file_name = output_name,
        ),
    ]

_pkgbuild = rule(
    implementation = _pkgbuild_impl,
    attrs = {
        "data": attr.label(
            mandatory = True,
            allow_single_file = [".tar"],
            doc = "A tar file that contains the data of the package.",
        ),
        "preinstall": attr.label(
            allow_single_file = True,
            doc = "Preinstall script",
        ),
        "postinstall": attr.label(
            allow_single_file = True,
            doc = "Postinstall script",
        ),
        "identifier": attr.string(mandatory = True, doc = "Identifier (reverse domain)."),
        "install_location": attr.string(default = "/", doc = "Install location."),
        "out": attr.output(mandatory = True),
        "version": attr.string(doc = "Package version."),
        "version_file": attr.label(allow_single_file = True, doc = "Package version."),
        "package_file_name": attr.string(doc = "Final package filename with variables substitution."),
        "package_variables": attr.label(
            doc = """Variables provider.""",
            providers = [PackageVariablesInfo],
        ),
        "_pkgbuild": attr.label(executable = True, cfg = "exec", default = Label("//bazel:pkgbuild")),
    },
    doc = "Use `pkgbuild` to generate a package for macOS.",
)

def pkgbuild(name, out = None, **kwargs):
    """@wraps(_pkgbuildb_impl)"""
    if not out:
        out = name + ".pkg"
    _pkgbuild(name = name, out = out, **kwargs)
