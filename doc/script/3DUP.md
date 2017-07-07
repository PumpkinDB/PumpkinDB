# 3DUP

{% method -%}

Duplicates the triplet of three topmost items

Input stack: `a bc `

Output stack: `a b c a b c`

{% common -%}

In this example, a triplet of three items (`0x00`, ``0x10`, `0x20`) is
duplicated.

```
PumpkinDB> 0x00 0x10 0x20 3DUP
0x00 0x10 0x20 0x00 0x10 0x20 
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items available on the stack

## Tests

```test
3dup_copies_a_pair : 0x00 0x10 0x20 3DUP 6 WRAP 0x010001100120010001100120 EQUAL?.
3dup_requires_three_items_0 : [3DUP] TRY UNWRAP 0x04 EQUAL?.
3dup_requires_three_items_1 : [1 3DUP] TRY UNWRAP 0x04 EQUAL?.
3dup_requires_three_items_2 : [1 2 3DUP] TRY UNWRAP 0x04 EQUAL?.
```