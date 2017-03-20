# SOME?

{% method -%}

Tests if the topmost stack item's length is larger than zero.

Input stack: `a`

Output stack: `c`

`SOME?` will push `1` if the item's length is larger than zero, `0` otherwise.

It has a "sister" instruction of [NONE?](NONEQ.md). Together they allow to
express a concept of an optional value.

{% common -%}

```
PumpkinDB> [] SOME?
0
PumpkinDB> [1] SOME?
1
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one items on the stack

## Tests

```test
works : [1] SOME?.
works_1 : [] SOME? NOT.
empty_stack : [SOME?] TRY UNWRAP 0x04 EQUAL?.
```
