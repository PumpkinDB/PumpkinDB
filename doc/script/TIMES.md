# TIMES

{% method -%}

Takes the topmost item and evaluates it as a PumpkinScript
program on the current stack `n` TIMES

Input stack: `code n`

Output stack: result of `code` evaluation done `n` times

`TIMES` evaluates the closure on a new stack and pops the previous
stack after each evaluation.

{% common -%}

```
PumpkinDB> [HLC] 3 TIMES
0x000014A2D295195171C80000 0x000014A2D295195211F00000 0x000014A2D29519526BC80000
```

{% endmethod %}

## Allocation

Allocates for recursion during runtime.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than two items on the stack

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
works : [10 >Q] 3 TIMES Q> Q> Q> STACK [10 10 10] EQUAL?.
works_0 : [10 >Q] 0 TIMES [Q>] TRY UNWRAP 0x0a EQUAL?.
empty_stack : [TIMES] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 TIMES] TRY UNWRAP 0x04 EQUAL?.
invalid_code : [1 1 TIMES] TRY UNWRAP 0x05 EQUAL?.
```
