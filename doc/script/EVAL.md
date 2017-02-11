# EVAL

Takes the topmost item and evaluates it as a PumpkinScript
program on the current stack

Input stack: `code`

Output stack: result of `code` evaluation

## Allocation

Allocates a copy of the code (this might change in the future)
during the runtime.

## Errors

EmptyStack error if there is less than one item on the stack

## Examples

```
10 [DUP] EVAL => 10 10
```

## Tests

```
10 [DUP] EVAL => 10 10
```