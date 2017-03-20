# NONE?

{% method -%}

Tests if the topmost stack item's length is equal to zero.

Input stack: `a`

Output stack: `c`

`NONE?` will push `1` if the item's length is equal to zero, `0` otherwise.

It has a "sister" instruction of [SOME?](SOMEQ.md). Together they allow to
express a concept of an optional value.

{% common -%}

```
PumpkinDB> [] NONE?
1
PumpkinDB> [1] NONE?
0
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one items on the stack

## Tests

```test
works : [] NONE?.
works_1 : [1] NONE? NOT.
empty_stack : [NONE?] TRY UNWRAP 0x04 EQUAL?.
```
