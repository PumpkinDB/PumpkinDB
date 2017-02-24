# 2NIP

{% method -%}

Drop the third and fourth items from the stack.

Input stack: `a b c d`

Output stack: `c d`

{% common -%}

```
PumpkinDB> 1 2 3 4 2NIP
3 4
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than four items on the stack

## Tests

```test
works : 1 2 3 4 2NIP STACK [3 4] EQUAL?.
empty_stack : [2NIP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 2NIP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 2NIP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_3 : [1 2 3 2NIP] TRY UNWRAP 0x04 EQUAL?.
```
