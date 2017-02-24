# 2DUP

{% method -%}

Duplicates the pair of two topmost items

Input stack: `a b`

Output stack: `a b a b`

{% common -%}

In this example, a pair of two items (`0x10` and `0x20`) is
duplicated.

```
PumpkinDB> 0x10 0x20 2DUP
0x10 0x20 0x10 0x20 
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items available on the stack

## Tests

```test
2dup_copies_a_pair : 0x10 0x20 2DUP 4 WRAP 0x10 0x20 OVER OVER 4 WRAP EQUAL?.
2dup_requires_two_items_0 : [2DUP] TRY UNWRAP 0x04 EQUAL?.
2dup_requires_two_items_1 : [1 2DUP] TRY UNWRAP 0x04 EQUAL?.
```