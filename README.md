[![Gitter chat](https://badges.gitter.im/PumpkinDB.png)](https://gitter.im/PumpkinDB/Lobby)
[![Build Status](https://travis-ci.org/PumpkinDB/PumpkinDB.svg?branch=master)](https://travis-ci.org/PumpkinDB/PumpkinDB)

PumpkinDB
=========

PumpkinDB is a compact event sourcing database, featuring fast on-disk storage,
flexible approach to event structure and encoding, sophisticated event indexing
and querying.

The core ideas behind PumpkinDB stem from the so called 
[lazy event sourcing](https://www.youtube.com/watch?v=aqv8d1pjmU8)
approach which is based on storing and indexing events while delaying domain
binding for as long as possible. That said, the intention of this database is to
be a building block for different kinds of event sourcing systems, ranging from
the classic one (using it as an event store) all the way to the lazy one (using
indices) and anywhere in between. It's also possible to implement different approaches
within a single database for different parts of the domain.

In previous incarnations (or, rather, inspirations) of PumpkinDB much more rigid structures,
formats and encoding were established as a prerequisite for using it, unnecessarily limiting
the applicability and appeal of the technology and ideas behind it. For example, one had to buy
into [ELF](https://rfc.eventsourcing.com/spec:1/ELF), UUID-based event identification and
[HLC-based](https://rfc.eventsourcing.com/spec:6/HLC) timestamps.

So it was deemed to be important to lift this kind of restrictions in PumpkinDB. But how do we
support all the formats without knowing what they are? What if there was a way to describe how data should be processed, for example,
for indexing — in a compact, unambiguous and composable form? Or even for recording data
itself?

Well, that's where the idea to use something like a Forth-like language was born.

Instead of devising custom protocols for talking to PumpkinDB, the protocol of communication has
become a pipeline to a script executor. This offers us enormous extension and flexibility capabilities.
 
To name a few:

* Low-level imperative querying (as a foundation for declarative queries)
* Indexing filters
* Subscription filters


PumpkinDB is in the process of active development and is not suitable for anything
beyond experimentation. The interfaces, guarantees and concepts will evolve over
time. You can read up on some of the ideas and progress in this repository's tlog
("gitlog"). Simply run `git log` and find commits with a lot of text in the message
and no diffs. 

So what **is** PumpkinDB?

* Fast file-based storage (thanks to [LMDB](http://lmdb.tech))
* PumpkinScript (Forth-like) language for data manipulation
* Library of building primitives: encoding, indexing, subscriptions, etc.
* Query language that compiles to PumpkinScript


## Trying it out

There are no releases at this time. You are welcome to clone the repository and run

```
$ cargo run --bin pumpkindb
...
Listening on 0.0.0.0:9981
```

You can connect to it using `pumpkindb-term`:

```
$ cargo run --bin pumpkindb-term
...
PumpkinDB> ["Hello" "world" ASSOC COMMIT] WRITE
PumpkinDB> ["Hello" RETR] READ
"world"
PumpkinDB>
```

(The above example stores key/value pair of "Hello" and "world" and
then retrieves the value associated with that key.)

You can change some of the server's parameters by creating `pumpkindb.toml`:

```toml
[storage]
path = "path/to/db"

[server]
port = 9981
```

## Contributing

This project is its very early days and we will always be welcoming
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
