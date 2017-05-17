# INT/EQUAL?

{% method -%}

Compares two topmost items for equality as signed big integers.

Input stack: `a b`

Output stack: `c`

`INT/EQUAL?` will push `1` if they are equal, `0` otherwise.

{% common -%}

```
PumpkinDB> 1 2 INT/EQUAL?
0
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be signed integers

## Tests

```test
not_equal : -1 +0 INT/EQUAL? NOT.
equal : +1000 +1000 INT/EQUAL?.
equal_diff_size : 0x8000000001 0x81 INT/EQUAL?.
requires_two_items_0 : [INT/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 INT/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
```
