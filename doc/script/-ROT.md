# -ROT

{% method -%}

Moves first item on the top to the third position

Input stack: `a b c`

Output stack: `c a b`

{% common -%}

```
PumpkinDB> 0x10 0x20 0x30 -ROT
0x30 0x10 0x20
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items on the stack

## Tests

```test
works : 1 2 3 -ROT 3 WRAP [3 1 2] EQUAL?.
empty_stack : [-ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 -ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 -ROT] TRY UNWRAP 0x04 EQUAL?.
```
