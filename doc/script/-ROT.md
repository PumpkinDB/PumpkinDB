# ROT

Moves first item on the top to the third position

Input stack: `a b c`

Output stack: `c a b`

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than three items on the stack

## Examples

```
0x10 0x20 0x30 -ROT => 0x30 0x10 0x20
```

## Tests

```test
works : 1 2 3 -ROT 3 WRAP [3 1 2] EQUAL?.
empty_stack : [-ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 -ROT] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [1 2 -ROT] TRY UNWRAP 0x04 EQUAL?.
```