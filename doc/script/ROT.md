# ROT

Moves third item from the top to the top

Input stack: `a b c`

Output stack: `b c a`

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than three items on the stack

## Examples

```
0x10 0x20 0x30 ROT => 0x20 0x30 0x10
```

## Tests

```test
works : 1 2 3 ROT 3 WRAP [2 3 1] EQUAL?.
empty_stack : [ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 ROT] TRY UNWRAP 0x04 EQUAL?.
```