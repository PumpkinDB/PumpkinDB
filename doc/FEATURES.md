# Experimental Features

PumpkinDB is new and many (experimental) features are being hashed out. Instead of
these features sitting in their own branches or Pull Requests, we decided to encourage
broader experimentation.

By default, no experimental features are enabled in a build. However,
one can enable all experimental features by building with an appropriate flag:

```
$ cargo build --features="experimental"
```

Or supply a space-delimited list of individual features of interest.

Once a feature is considered to be stable enough, the feature can be
first promoted to the `default` feature set and once fully graduated
(after a period of additional testing received through the inclusion
in`default`), the feature gate can be dropped.

## Current experimental features

## Graduated features

Graduated features are enabled by default, but in the source code,
they are still behind a feature gate. This means that if things go
wrong, they can still be easily demoted or dropped altogether. If
everything is good, though, the gate will be eventually dropped.

* `scoped_dictionary` ([issue #71](https://github.com/PumpkinDB/PumpkinDB/issues/71))