# UINT/EQUAL?

{% method -%}

Compares two topmost items for equality as unsigned big integers.

Input stack: `a b`

Output stack: `c`

`UINT/EQUAL?` will push `1` if they are equal, `0` otherwise.

{% common -%}

```
PumpkinDB> 1 2 UINT/EQUAL?
0
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
not_equal : 1 0 UINT/EQUAL? NOT.
equal : 1000 1000 UINT/EQUAL?.
equal_diff_size : 0x01 0x0001 UINT/EQUAL?.
requires_two_items_0 : [UINT/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 UINT/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
```
