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

[InvalidValue](../ERRORS/InvalidValue.md) error if `a` is less than `b`

## Examples

```
2 1 UINT/SUB => 1
```

## Tests

```test
works : 2 1 UINT/SUB 1 EQUAL?.
invalid_value : [1 2 UINT/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [UINT/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 UINT/SUB] TRY UNWRAP 0x04 EQUAL?.
```