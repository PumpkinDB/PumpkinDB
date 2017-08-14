# PumpkinDB


[![Gitter chat](https://badges.gitter.im/PumpkinDB.png)](https://gitter.im/PumpkinDB/Lobby)
[![Code Triagers](https://www.codetriage.com/pumpkindb/pumpkindb/badges/users.svg)](https://www.codetriage.com/pumpkindb/pumpkindb)
[![OpenCollective](https://opencollective.com/pumpkindb/backers/badge.svg)](#backers)
[![OpenCollective](https://opencollective.com/pumpkindb/sponsors/badge.svg)](#sponsors)

| | |
|-|-|
| Build status (Linux) | [![Build Status](https://travis-ci.org/PumpkinDB/PumpkinDB.svg?branch=master)](https://travis-ci.org/PumpkinDB/PumpkinDB) |
| Build status (Windows) | [![Windows Build status](https://ci.appveyor.com/api/projects/status/picau5286hr9ynl7?svg=true)](https://ci.appveyor.com/project/yrashk/pumpkindb) |
| Project status | Usable, between alpha and beta |
| Production-readiness | Depends on your risk tolerance |

PumpkinDB is an immutable ordered key-value database engine, featuring:

* ACID transactions
* Persistent storage
* An embedded programming language (PumpkinScript)
* Binary keys and values (allows any encoding to be used: JSON, XML, Protobuf, Cap'n Proto, etc.)
* Standalone and embedded scenarios

## Why immutable?

Simply put, the data replaced is data deleted and is therefore, an unsafe way to manage data. Bugs,
misunderstanding, changing scope and requirements and other factors might influence what data (and
especially *past* data) means and how can it be used.

By guaranteeing the immutability of key's value once it is set, PumpkinDB forces its users
to think of their data through a temporal perspective.

This approach is highly beneficial for implementing event sourcing and similar types of architectures.

## What is PumpkinDB?

PumpkinDB is essentially a database programming environment, largely inspired by core ideas behind [MUMPS](https://en.wikipedia.org/wiki/MUMPS). Instead of M,
it has a Forth-inspired stack-based language, PumpkinScript. Instead of hierarchical keys, it has a flat key namespace and doesn't allow overriding values once they are set.  Core motivation for immutability was that with the cost of storage declining, erasing data is effectively a strategical mistake.

While not intended for general purpose programming, its main objective is to facilitate building specialized application-specific and generic databases with a particular focus on immutability and processing data as close to storage as possible, incurring as little communication penalty as possible.

Applications communicate with PumpkinDB by sending small PumpkinScript programs
over a network interface (or API when using PumpkinDB as an embedded solution).

PumpkinDB offers a wide array of primitives for concurrency, storage, journalling, indexing and other common building blocks.

## Why is it a database engine?

The core ideas behind PumpkinDB stem from the so called
[lazy event sourcing](https://www.youtube.com/watch?v=aqv8d1pjmU8)
approach which is based on storing and indexing events while delaying domain
binding for as long as possible. That said, the intention of this database is to
be a building block for different kinds of architectures, be it
classic event sourcing (using it as an event store), lazy event sourcing (using
indices) or anything else. It's also possible to implement different approaches within
a single database for different parts of the domain.

Instead of devising custom protocols for talking to PumpkinDB, the protocol of
communication has become a pipeline to a script executor. This offers us enormous extension
and flexibility capabilities.

While an external application can talk to PumpkinDB over a network connection, PumpkinDB's
engine itself is embeddable and can be used directly. Currenly, it is available for Rust
applications only, but this may one day extend to all languages that can interface with C.

## Client libraries

| Language | Library | Status |
|----------|---------|--------|
| **Rust** | [pumpkindb_client](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_client) | Early release ([0.2.0](https://crates.io/crates/pumpkindb_client/0.2.0)) |
| **Java** | [pumpkindb-client](https://github.com/PumpkinDB/pumpkindb-java) | Pre-release |

## Trying it out

You can download PumpkinDB releases [from GitHub](https://github.com/PumpkinDB/PumpkinDB/releases).

### Docker

You can try out latest PumpkinDB HEAD revision by using a docker image:

```shell
$ docker pull pumpkindb/pumpkindb
```

Alternatively, you can build the image yourself:

```shell
$ docker build . -t pumpkindb/pumpkindb
```

Run the server:

```shell
$ docker run -p 9981:9981 -ti pumpkindb/pumpkindb
2017-04-12T02:52:47.440873517+00:00 WARN pumpkindb - No logging configuration specified, switching to console logging
2017-04-12T02:52:47.440983318+00:00 INFO pumpkindb - Starting up
2017-04-12T02:52:47.441122740+00:00 INFO pumpkindb_engine::storage - Available disk space is approx. 56Gb, setting database map size to it
2017-04-12T02:52:47.441460231+00:00 INFO pumpkindb - Starting 4 schedulers
2017-04-12T02:52:47.442375937+00:00 INFO pumpkindb - Listening on 0.0.0.0:9981
```

Finally, connect to it using `pumpkindb-term`:

```
$ docker run -ti pumpkindb/pumpkindb pumpkindb-term 172.17.0.1:9981 # replace IP with the docker host IP
```

### Building from the source code

You are also welcome to clone the repository and build
it yourself. You will need Rust Nightly to do this. The easiest way to get it is to use
[rustup](https://www.rust-lang.org/en-US/install.html)

```shell
$ rustup install nightly
$ rustup override set nightly # in PumpkinDB directory
```

After that, you can run PumpkinDB server this way:

```shell
$ cargo build --all
$ ./target/debug/pumpkindb
2017-04-03T10:43:49.667667-07:00 WARN pumpkindb - No logging configuration specified, switching to console logging
2017-04-03T10:43:49.668660-07:00 INFO pumpkindb - Starting up
2017-04-03T10:43:49.674139-07:00 INFO pumpkindb_engine::storage - Available disk space is approx. 7Gb, setting database map size to it
2017-04-03T10:43:49.675759-07:00 INFO pumpkindb - Starting 8 schedulers
2017-04-03T10:43:49.676113-07:00 INFO pumpkindb - Listening on 0.0.0.0:9981
```

You can connect to it using `pumpkindb-term`:

```shell
$ ./target/debug/pumpkindb-term
Connected to PumpkinDB at 0.0.0.0:9981
To send an expression, end it with `.`
Type \h for help.
PumpkinDB> ["Name" HLC CONCAT "Jopn Doe" ASSOC COMMIT] WRITE.

PumpkinDB> ["Name" HLC CONCAT "John Doe" ASSOC COMMIT] WRITE.

PumpkinDB> [CURSOR DUP "Name" CURSOR/SEEKLAST DROP CURSOR/VAL] READ (Get last value).
"John Doe"
PumpkinDB> [CURSOR DUP "Name" CURSOR/SEEKLAST DROP DUP CURSOR/PREV DROP CURSOR/VAL] READ (Get previous value).
"Jopn Doe"
```

(The above example shows how one can query and navigate for values submitted at a different time, using low level primitives).

You can change some of the server's parameters by creating `pumpkindb.toml`:

```toml
[storage]
path = "path/to/db"
# By default, mapsize will equal to the size of
# available space on the disk, except on Windows,
# where default would be 1Gb.
# `mapsize` is a theoretical limit the database can
# grow to. However, on Windows, this also means that
# the database file will take that space.
# This parameter allows to specify the mapsize
# in megabytes.
# mapsize = 2048

[server]
port = 9981
```


## Components

PumpkinDB project is split into a couple of separate components (crates):

* [pumpkinscript](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkinscript) — PumpkinScript parser. Allows to convert text PumpkinScript form into binary one.
* [pumpkindb_engine](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_engine) — Core PumpkinDB library. Provides PumpkinScript scheduler, and a standard library of instructions
* [pumpkindb_mio_server](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_mio_server) — Async MIO-based PumpkinDB server library. Useful for building custom PumpkinProtocol-compatible servers.
* [pumpkindb_client](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_client) — PumpkinProtocol client library.
* [pumpkindb_server](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_server) — Stock PumpkinDB server. Built on top of `pumpkindb_mio_server`.
* [pumpkindb_term](https://github.com/PumpkinDB/PumpkinDB/tree/master/pumpkindb_term) — console-based PumpkinDB server client.
* [doctests](https://github.com/PumpkinDB/PumpkinDB/tree/master/tests/doctests) — a small utility to run instructions doctests.

## Contributing

This project is in its very early days and we will always be welcoming
contributors.

Our goal is to encourage frictionless contributions to the project. In order to
achieve that, we use Unprotocols [C4 process](https://rfc.unprotocols.org/spec:1/C4).
Please read it, it will answer a lot of questions. Our goal is to merge pull requests
as quickly as possible and make new stable releases regularly.

In a nutshell, this means:

* We merge pull requests rapidly (try!)
* We are open to diverse ideas
* We prefer code now over consensus later

To learn more, read our [contribution guidelines](CONTRIBUTING.md)

We also maintain a [list of issues](https://github.com/PumpkinDB/PumpkinDB/issues?q=is%3Aissue+is%3Aopen+label%3AWhatCanIStartWith%3F) that we think are good starters for new
contributors.

## Backers

Support us with a monthly donation and help us continue our activities. [[Become a backer](https://opencollective.com/pumpkindb#backer)]

<a href="https://opencollective.com/pumpkindb/backer/0/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/0/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/1/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/1/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/2/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/2/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/3/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/3/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/4/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/4/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/5/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/5/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/6/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/6/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/7/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/7/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/8/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/8/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/9/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/9/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/10/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/10/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/11/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/11/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/12/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/12/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/13/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/13/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/14/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/14/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/15/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/15/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/16/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/16/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/17/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/17/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/18/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/18/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/19/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/19/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/20/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/20/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/21/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/21/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/22/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/22/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/23/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/23/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/24/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/24/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/25/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/25/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/26/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/26/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/27/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/27/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/28/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/28/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/backer/29/website" target="_blank"><img src="https://opencollective.com/pumpkindb/backer/29/avatar.svg"></a>

## Sponsors

Become a sponsor and get your logo on our README on Github with a link to your site. [[Become a sponsor](https://opencollective.com/pumpkindb#sponsor)]

<a href="https://opencollective.com/pumpkindb/sponsor/0/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/0/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/1/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/1/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/2/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/2/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/3/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/3/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/4/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/4/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/5/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/5/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/6/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/6/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/7/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/7/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/8/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/8/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/9/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/9/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/10/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/10/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/11/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/11/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/12/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/12/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/13/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/13/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/14/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/14/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/15/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/15/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/16/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/16/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/17/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/17/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/18/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/18/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/19/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/19/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/20/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/20/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/21/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/21/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/22/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/22/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/23/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/23/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/24/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/24/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/25/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/25/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/26/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/26/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/27/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/27/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/28/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/28/avatar.svg"></a>
<a href="https://opencollective.com/pumpkindb/sponsor/29/website" target="_blank"><img src="https://opencollective.com/pumpkindb/sponsor/29/avatar.svg"></a>
