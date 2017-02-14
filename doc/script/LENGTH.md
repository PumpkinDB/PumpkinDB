# LENGTH

Puts the length of the top item on the stack back to the top of the stack

Input stack: `a`

Output stack: `b`

`LENGTH` pops a top item off the stack and pushes its length back to the
top of the stack.

## Allocation

Allocates for the result of the item size calculation

## Errors

EmptyStack error if there are no items on the stack

## Examples

```
"Hello" LENGTH => 5
```

## Tests

```
[] LENGTH => 0
"Hello" LENGTH => 5
```