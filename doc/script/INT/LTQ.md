# INT/LT?

{% method -%}

Compares two topmost items as signed big integers.

Input stack: `a b`

Output stack: `a`

`INT/LT?` will push `1` if `a` is strictly less than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 INT/LT?
1
PumpkinDB> 0x20 0x10 INT/LT?
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
invalid : [0xff 0xff INT/LT?] TRY UNWRAP 0x03 EQUAL?.
less : +1 +2 INT/LT?.
less_diff_size : 0x010001 0x0110 INT/LT?.
greater : +2 +1 INT/LT? NOT.
equal : +1 +1 INT/LT? NOT.
requires_two_items_0 : [INT/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 INT/LT?] TRY UNWRAP 0x04 EQUAL?.
```
