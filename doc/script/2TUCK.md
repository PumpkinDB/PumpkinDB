# 2TUCK

{% method -%}

Copy the top pair of items below the second pair of items.

Input stack: `a b c d`

Output stack: `c d a b c d`

{% common -%}

```
PumpkinDB> 1 2 3 4 2TUCK
3 4 1 2 3 4
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than four items on the stack

## Tests

```test
works : 1 2 3 4 2TUCK STACK [3 4 1 2 3 4] EQUAL?.
empty_stack : [2TUCK] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 2TUCK] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 2TUCK] TRY UNWRAP 0x04 EQUAL?.
empty_stack_3 : [1 2 3 2TUCK] TRY UNWRAP 0x04 EQUAL?.
```
