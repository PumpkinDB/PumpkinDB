# 2OVER

{% method -%}

Copies the second topmost pair of items to the top of the stack

Input stack: `a b c d`

Output stack: `a b c d a b`

{% common -%}

```
PumpkinDB> 1 2 3 4 2OVER
1 2 3 4 1 2
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than four items on the stack

## Tests

```test
works : 1 2 3 4 2OVER STACK [1 2 3 4 1 2] EQUAL?.
empty_stack : [2OVER] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 2OVER] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 2OVER] TRY UNWRAP 0x04 EQUAL?.
empty_stack_3 : [1 2 3 2OVER] TRY UNWRAP 0x04 EQUAL?.
```
