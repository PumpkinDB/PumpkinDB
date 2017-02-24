# OVER

{% method -%}

Copies the second topmost item to the top of the stack

Input stack: `a b`

Output stack: `a b a`

{% common -%}

```
PumpkinDB> 0x10 0x20 OVER
0x10 0x20 0x10
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : 1 2 OVER 3 WRAP [1 2 1] EQUAL?.
requires_two_items_0 : [OVER] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 OVER] TRY UNWRAP 0x04 EQUAL?.
```
