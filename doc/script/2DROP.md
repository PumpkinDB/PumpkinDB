# 2DROP

{% method -%}

Drops two topmost items off the top of the stack

Input stack: `a b`

Output stack: -

{% common -%}

In this example, two items (`1` and `2`) are dropped from the stack,
leaving the third element on the top of the stack.

```
PumpkinDB> 0 1 2 2DROP
0
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items available on the stack

## Tests

```test
2drop_drops_two_items : 1 2 2DROP STACK LENGTH 0 EQUAL?.
2drop_requires_two_items_0 : [2DROP] TRY UNWRAP 0x04 EQUAL?.
2drop_requires_two_items_1 : [1 2DROP] TRY UNWRAP 0x04 EQUAL?.
```