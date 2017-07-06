# 3DROP

{% method -%}

Drops three topmost items off the top of the stack

Input stack: `a b c`

Output stack: -

{% common -%}

In this example, three items (`0`, `1` and `2`) are dropped from the stack,
leaving the forth element on the top of the stack.

```
PumpkinDB> 0 0 1 2 3DROP
0
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items available on the stack

## Tests

```test
3drop_drops_three_items : 1 2 3 3DROP STACK LENGTH 0 EQUAL?.
3drop_requires_three_items_0 : [3DROP] TRY UNWRAP 0x04 EQUAL?.
3drop_requires_three_items_1 : [2 3DROP] TRY UNWRAP 0x04 EQUAL?.
3drop_requires_three_items_2 : [1 2 3DROP] TRY UNWRAP 0x04 EQUAL?.
```