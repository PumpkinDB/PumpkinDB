# UINT/GT?

{% method -%}

Compares two topmost items as unsigned big integers.

Input stack: `a b`

Output stack: `a`

`UINT/GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 UINT/GT?
0
PumpkinDB> 0x20 0x10 UINT/GT?
1
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
less : 0x10 0x20 UINT/GT? NOT.
greater : 0x20 0x10 UINT/GT?.
greater_diff_size : 0x0020 0x10 UINT/GT?.
equal : 0x10 0x10 UINT/GT? NOT.
requires_two_items_0 : [UINT/GT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 UINT/GT?] TRY UNWRAP 0x04 EQUAL?.
```
