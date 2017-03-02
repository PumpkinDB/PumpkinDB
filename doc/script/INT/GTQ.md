# INT/GT?

{% method -%}

Compares two topmost items as signed big integers.

Input stack: `a b`

Output stack: `a`

`INT/GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 INT/GT?
0
PumpkinDB> 0x20 0x10 INT/GT?
1
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be signed integers

## Tests

```test
invalid : [0xff 0xff INT/GT?] TRY UNWRAP 0x03 EQUAL?.
less : +1 +2 INT/GT? NOT.
greater : +2 +1 INT/GT?.
greater_diff_size : 0x010002 0x0101 INT/GT?.
equal : +1 +1 INT/GT? NOT.
requires_two_items_0 : [INT/GT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 INT/GT?] TRY UNWRAP 0x04 EQUAL?.
```
