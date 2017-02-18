SUB
===

Subtracts one unsigned integer from another

Input stack: `a` `b`
Output stack: `c`

`SUB` will subtract of `b` from `a` and push it to the top of the stack.

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
2 1 UINT/SUB => 1
```

## Tests

```
2 1 UINT/SUB => 1
```