# EVAL

{% method -%}

Takes the topmost item and evaluates it as a PumpkinScript
program on the current stack

Input stack: `code`

Output stack: result of `code` evaluation

`EVAL` evaluates the closure on the current stack.

{% common -%}

```
PumpkinDB> 10 [DUP] EVAL
10 10
```

{% endmethod %}

## Allocation

Allocates a copy of the code (this might change in the future)
during the runtime.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
works : 10 [DUP] EVAL 2 WRAP 10 10 2 WRAP EQUAL?.
empty_stack : [EVAL] TRY UNWRAP 0x04 EQUAL?.
invalid_code : [1 EVAL] TRY UNWRAP 0x05 EQUAL?.
```
