# TRY

Takes the topmost item and safely evaluates it as a PumpkinScript
program on the current stack

Input stack: `code`
Output stack: `[]` or `[description details code]` (error closure)

`TRY` is a close relative of [EVAL](EVAL.md). It also evaluates
the closure but will not fail the program if there was an error.
Instead, it will push an error closure onto the stack. If no error
occurred, `[]` (an empty closure) will be pushed onto the stack.

## Allocation

Allocates a copy of the code (this might change in the future)
during the runtime. Allocates on program's heap when recovering
from an error that occurred.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than one item on the stack

## Examples

```
[DUP] TRY SOME? => 0x1
[1 DUP] TRY SOME? => 0x1 0x1 0x0
```

## Tests

```
[DUP] TRY SOME? => 0x1
[1 DUP] TRY SOME? => 0x1 0x1 0x0
```