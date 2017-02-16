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
0x10 0x20 0x30 SWAP => 0x20 0x30 0x10
```

## Tests

```
0x10 0x20 0x30 SWAP => 0x20 0x30 0x10
```