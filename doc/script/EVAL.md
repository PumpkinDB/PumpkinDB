# EVAL

Takes the topmost item and evaluates it as a PumpkinScript
program on the current stack

Input stack: `code`

Output stack: result of `code` evaluation

## Allocation

Allocates a copy of the code (this might change in the future)
during the runtime.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than one item on the stack

[Decoding error](./ERRORS/DECODING.md) error if the code is undecodable.

## Examples

```
10 [DUP] EVAL => 10 10
```

## Tests

```test
works : 10 [DUP] EVAL 2 WRAP 10 10 2 WRAP EQUAL?.
empty_stack : [EVAL] TRY UNWRAP 0x04 EQUAL?.
invalid_code : [1 EVAL] TRY UNWRAP 0x05 EQUAL?.
```