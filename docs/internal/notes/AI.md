# Documented AI usage and attempts

**Model used**: Claude Sonnet 4.6 Thinking via Perplexity

## Naming things
**Issue**: While I know the concepts, I don't know their correct names, or
sometimes I struggle to find distinct names. Additionaly, having to switch
between french and english makes things harder.

**Prior work**: *not applicable*

**Methodology**: Describing the term or concept, or the issue with the current
term and asking for other names.

**Expected results**:
1. AI SHOULD find good names for what I'm describing.

**Actual results**:
1. AI DID find good names for what I was describing

**Details**: *not applicable*

**Conclusion**: AI did help and saved some time (unable to estimate).

## Find missing Linux kernel configuration options
**Issue**: Podman wasn't able to launch containers correctly because the
`defconfig` does not contain all the required features.

**Prior work**: Most flag were found manually but once it got to the netavark
setup, the error messages weren't helpful enough to find the flags.

**Methodology**: Giving my Kernel config fragment (with only changes from
defconfig), the error message, the context (`defconfig`, embedded system,
etc.), and asking to either give the missing flag, or find sources.

**Expected results**:
1. AI SHOULD find the missing flag, or;
2. AI SHOULD point me to a direction.

**Actual results**:
1. AI DID NOT find the missing flag, and;
2. AI DID point me to a direction.

**Details**: AI did not find the missing flag, but it pointed me to Gentoo's
wiki from where I found another page with some missing flags. The only remaining
flag turned out to be `CONFIG_NFT_FIB_IPV6`. The AI did not surface this one,
nor did it surface a source. After AI failed to find the flag, I re-researched
everything and stumbled on https://github.com/containers/podman-compose/issues/1154#issuecomment-3281224387
which gave the missing flag.

**Conclusion**: AI did not help and was a waste of time (estimated 1h30).

## Improve Nix build pipeline for Rust
**Issue**: Using Nix and Crane to build the Rust workspace, everytime any of the
crate changed, all crates were rebuilt (as opposed to only those dependent on
the crate).

**Prior work**: Reading Crane's documentation and stubbornly ignoring its
recommendations.

**Methodology**: Giving the relevant Nix files and the workspace Cargo.toml,
describing the issue, and asking to produce a corrected Nix file.

**Expected results**:
1. AI SHOULD find the root cause of the issue, and;
2. AI SHOULD propose a working solution

**Actual results**:
1. AI DID NOT find the root cause, and;
2. AI DID NOT propose a working solution

**Details**: Because I had put *every* dependencies (including local ones) in
the workspace's `Cargo.toml`, the crane artifact preparation was always
invalidated. Removing them from the workspace and instead manually specifying
the path in each dependent crate worked. It required the addition of a `deps`
options in the build function to include the crates in the source.

**Conclusion**: AI did not help and was a waste of time (estimated 30mn).

## Setup Rust namespaced integration tests
**Issue**: Due to the nature of the packages and the fact that they interact
with the Linux kernel, writing unit test or integration tests is tricky: they
will modify the "host" kernel and may break things, e.g. adding/removing
interfaces, modifying `/etc/resolv.conf`, etc.

**Prior work**: *none*

**Methodology**: Describing the issue and asking to find relevant crates.

**Expected results**:
1. AI SHOULD point me to a crate that I can use to create isolated tests.

**Actual results**:
1. AI DID point me to a crate (https://github.com/canndrew/netsim), while not *exactly* what I needed, I was able to
   take inspiration in the source code to produce my own version.

**Conclusion**: AI did help and saved some time (estimated 15mn).
