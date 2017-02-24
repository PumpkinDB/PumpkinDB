# 2ROT

{% method -%}

Rotate the top three pairs on the stack bringing pair `a b` to the top of the stack.

Input stack: `a b c d e f`

Output stack: `c d e f a b`

{% common -%}

```
PumpkinDB> 1 2 3 4 5 6 2ROT
3 4 5 6 1 2
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than six items on the stack

## Tests

```test
works : 1 2 3 4 5 6 2ROT STACK [3 4 5 6 1 2] EQUAL?.
empty_stack : [2ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 2ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 2ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_3 : [1 2 3 2ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_4 : [1 2 3 4 2ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_5 : [1 2 3 4 5 2ROT] TRY UNWRAP 0x04 EQUAL?.
```
