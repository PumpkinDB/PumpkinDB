# 0.2.0

In this release, the most notable change was splitting of PumpkinDB into
multiple crates. This enables PumpkinDB users to build custom systems
(embedded and client-server ones) that use PumpkinDB as an engine. Also,
pumpkinscript and pumpkindb_client crates allow building Rust applications
that use PumpkinDB as remote clients. pumpkindb_mio_server crate simplifies
developing custom PumpkinDB server builds.

A confusing "word" term has been changed to "instruction" to provide a clearer
indication of the concept behind it.

We've also introduced more conventions around numbers (especially
signed and unsigned sized integers, as well as floats) and added respective
instructions to the library.

Initial UUID support has been added. Cursor API surface has been reduced and
is much easier to comprehend now.

Internally, a lot has changed. A new messaging system (it is now available
not only through the server component, but pumpkindb_engine as well). Dispatcher
trait allows defining custom sets of instruction "modules" and is flexible enough
to support both static and dynamic dispatching.

And, of course, a number of bugs has been spotted and some of them were even
fixed. We've also optimized a number of things and found out that there's a lot
more to optimize in the upcoming versions!

This is the first release with published [library crates on crates.io](https://crates.io/search?q=pumpkindb)

Also, this release includes statically built versions of PumpkinDB server
and terminal for Linux (using musl). This is an experimental build as in
the future we might need to depend on some dynamically libraries. 

New contributors: alex, Christoph Herzog, Guillaume Gauvrit, Matteo Semenzato
and Stuart Hinson.

Thank you!

SHA-256 checksums:

| File | Checksum |
|------|----------|
| pumpkindb-v0.2.0-x86_64-apple-darwin.tar.gz | f7e81dfa0bfc02de31b4bd70c5b6a13e95a44550d33d53a77179f5c23a9efde4 |
| pumpkindb-v0.2.0-x86_64-pc-windows-msvc.zip | cf4b0fb7e893fdcd4fc7881b01e7c4bf045594e85bdd73cc88d2fec8ac2418a1  |
| pumpkindb-v0.2.0-x86_64-unknown-linux-gnu.tar.gz | efcca81f4b482d6a69bfe08f935d2da61c8f7e2262798dcd93a656aeb8c58e15 |
| pumpkindb-v0.2.0-x86_64-unknown-linux-musl.tar.gz | 4dc32dbf05dbe0df727128c7455c5482d3366ebf1a32f7308d22120a53c67388 |

# 0.1.0

This is PumpkinDB's first release (ever!), the intention of which is to finally
provide some binaries to those who want to play with PumpkinDB but can't be
bothered to install Rust and switch to nightly builds just yet!

Don't expect it to be very stable or feature-complete. Those were the non-goals
this time around!

We'd really like you to try it out and let us know what you think about it!

SHA-256 checksums:

| File | Checksum |
|------|----------|
| pumpkindb-v0.1.0-x86_64-apple-darwin.tar.gz | 0c4bf2f56bee2139d3a0616be882df828fe881abd45a8176d076547fb9da7ea8 |
| pumpkindb-v0.1.0-x86_64-pc-windows-msvc.zip | f8d0f48486ef790ee36620823503aaefaa6a4eb9fef856b136e948791318d2d4 |
| pumpkindb-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | 5d1355cf88a7b691c12ad8ec2bca3cbf783e42cde48a975b059890ed88496fad |
