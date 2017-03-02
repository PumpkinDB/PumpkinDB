# UINT/LT?

{% method -%}

Compares two topmost items as unsigned big integers.

Input stack: `a b`

Output stack: `a`

`UINT/LT?` will push `1` if `a` is strictly less than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 UINT/LT?
1
PumpkinDB> 0x20 0x10 UINT/LT?
0
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
less : 0x10 0x20 UINT/LT?.
less_diff_size : 0x0010 0x20 UINT/LT?.
greater : 0x20 0x10 UINT/LT? NOT.
equal : 0x10 0x10 UINT/LT? NOT.
requires_two_items_0 : [UINT/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 UINT/LT?] TRY UNWRAP 0x04 EQUAL?.
```
