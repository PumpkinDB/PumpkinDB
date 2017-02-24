# ROT

{% method -%}

Moves third item from the top to the top

Input stack: `a b c`

Output stack: `b c a`

{% common -%}

```
PumpkinDB> 0x10 0x20 0x30 ROT
0x20 0x30 0x10
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items on the stack

## Tests

```test
works : 1 2 3 ROT 3 WRAP [2 3 1] EQUAL?.
empty_stack : [ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 ROT] TRY UNWRAP 0x04 EQUAL?.
```
