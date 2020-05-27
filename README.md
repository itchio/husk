# husk

Provides APIs useful for [itch](https://github.com/itchio/itch) and
[itch-setup](https://github.com/itchio/itch-setup).

husk provides:

  * A Rust interface (this crate, as a "lib")
  * A C interface (this crate, as a "staticlib"), with a header generated with `cbindgen`
  * A Go interface, with cgo bindings generated `c-for-go` (`lowhusk`), and
  higher-level bindings written by hand (`husk`).

See the build script in `release/build.js`.

## Intent

The idea is, at first, to be able to let `libbutler` (see
[valet](https://github.com/itchio/valet)) use components written in Rust,
like shortcut creation on Windows with proper Unicode handling, using
the `IShellLink` COM interface.

Over time, more and more code from `libbutler` could be migrated to either
`husk` or `valet`. For example, `valet` could handle all the JSON-RPC
messaging, and `husk` could handle the itch.io REST API calls, downloads,
extraction, sandboxing, etc.

Eventually, `libbutler` could be completely retired - connecting `valet`
and `husk` directly. Which should be relatively effortless, as `husk`
already exposes a Rust interface.

## Using husk from Go

...is awkward. Packages that use husk will never be `go get`-able.

husk first needs to be built and installed somewhere - but it links statically,
so at least the headache is only at build time.

The idea right now is to build binaries in CI, upload them to GitHub
releases, and have the build scripts for `itch-setup` and `libbutler`
download the binaries to a `vendor/` directory or something, with replace
directives in their `go.mod`, like the sample Go binary in this repo has.
