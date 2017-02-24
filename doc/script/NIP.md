# NIP

{% method -%}

Drop the first item below the top of stack.

Input stack: `a b`

Output stack: `b`

{% common -%}

```
PumpkinDB> 0x10 0x20 NIP
0x20
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : 1 2 NIP 2 EQUAL?.
empty_stack : [NIP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 NIP] TRY UNWRAP 0x04 EQUAL?.
```
