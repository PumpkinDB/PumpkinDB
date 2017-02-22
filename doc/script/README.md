# PumpkinScript

PumpkinScript is a minimalistic concatenative, stack-based language inspired
by Forth.

It is used in PumpkinDB to operate a low-level database "virtual machine" —
to manipulate, record and retrieve data.

This is an ultimate gateway to flexibility in terms of how PumpkinDB can operate, what
formats can it support, etc.

## Types

PumpkinScript has no types, all values on the stack are byte arrays. However,
there are some conventions:

* Big integers represented as (unlimited length) big-endian byte arrays
* Strings are represented as UTF-8 encoded byte arrays

## Text form

While internally (and over the optimized wire) PumpkinScript is represented
in a binary form, there is a text form that's easy for people to read
and write.

The format is simple, it is a sequence of space-separated tokens,
with binaries represented as:

* `0x<hexadecimal>` (hexadecimal form)
* `"STRING"` (string form, no quoted characters support yet)
* `integer` (integer form, will convert to a big endian big integer)
* `'word` (word in a binary form)

The rest of the instructions considered to be words.

Example: `"Hello" 0x20 "world" CONCAT CONCAT`

One additional piece of syntax is code included within square
brackets: `[DUP]`. This means that the parser will take the code inside,
compile it to the binary form and add as a data push. This is useful for
words like [EVAL](EVAL.md). Inside of this syntax, you can use so-called "unwrapping"
syntax (\`word) that can embed a value of a word into this code:

```test
unwrapping : 1 'a SET [`a] 'b SET 2 'a SET b EVAL 0x01 EQUAL?.
```

(this verifies that the closure we save in `b` remains at 1
while we re-set `a` to 2)

