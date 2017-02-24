# TUCK

{% method -%}

Copy the top stack item below the second stack item.

Input stack: `a b`

Output stack: `b a b`

{% common -%}

```
PumpkinDB> 0x10 0x20 TUCK
0x20 0x10 0x20
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : 1 2 TUCK STACK [2 1 2] EQUAL?.
empty_stack : [TUCK] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 TUCK] TRY UNWRAP 0x04 EQUAL?.
```
