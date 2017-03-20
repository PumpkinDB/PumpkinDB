# PumpkinScript

PumpkinScript is a minimalistic concatenative, stack-based language inspired
by Forth. It is used in PumpkinDB to operate a low-level database "virtual machine" —
to manipulate, record and retrieve data. This is an ultimate gateway to flexibility in terms of how PumpkinDB can operate, what
formats can it support, etc.

PumpkinDB has also been influenced by ideas found in [MUMPS](https://en.wikipedia.org/wiki/MUMPS)
and PumpkinScript shares certain similarities with M, mostly aiming at being a practical
data processing language and being a little bit quirky and imperfect.

## Types

PumpkinScript has no types, all values on the stack are byte arrays. However,
there are some conventions:

* Unsigned big integers represented as (unlimited length) big-endian byte arrays
* Strings are represented as UTF-8 encoded byte arrays

## Text form

While internally (and over the optimized wire) PumpkinScript is represented
in a binary form, there is a text form that's easy for people to read
and write. The format is simple, it is a sequence of space-separated tokens,
with binaries represented as:

{% method -%}
* `0x<hexadecimal>` (hexadecimal form)
* `"STRING"` (string form, newline and double quotes can be escaped with `\`)
* `integer` (integer form, will convert to a big endian big integer)
* `'instruction` (instruction in a binary form)

The rest of the instructions considered to be instructions.

{% common -%}

`"Hello" 0x20 1 "world" CONCAT CONCAT 'CONCAT`

{% endmethod %}


{% method -%}
One additional piece of syntax is code included within square
brackets: `[DUP]`. This means that the parser will take the code inside,
compile it to the binary form and add as a data push. This is useful for
instructions like [EVAL](EVAL.md).
 
Inside of this syntax, you can use so-called "unwrapping"
syntax (``instruction`) that can embed a value of a instruction into this code. 
The reason for this syntax is quite important: by default, every instruction
included into this block of code is passed by name. However, there
 are cases when it is important to pass instruction's value instead, as
 instruction values can change or be overridden in scoped evaluations.

{% common -%}

In this example, we are passing `DUP` script on the stack for
`IF` to execute if previous value on the stack is truthy (1).

```
[DUP] IF
```

Unwrapping is done this way:

```test
unwrapping : 1 'a SET [`a] 'b SET 2 'a SET b EVAL 0x01 EQUAL?.
```

It is also possible to unwrap multiple levels:

```test
unwrapping_multiple : 1 'a SET [[``a] EVAL] 'b SET 2 'a SET b EVAL 0x01 EQUAL?.
```

This verifies that the closure we save in `b` remains at 1
while we re-set `a` to 2.

{% endmethod %}


