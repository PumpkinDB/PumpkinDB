# NIP

Drop the first item below the top of stack.

Input stack: `a b`

Output stack: `b`

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
0x10 0x20 NIP => 0x20
```

## Tests

```test
works : 1 2 NIP 2 EQUAL?.
empty_stack : [NIP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 NIP] TRY UNWRAP 0x04 EQUAL?.
```