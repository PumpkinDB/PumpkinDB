# 2DUP

Duplicates the pair of two topmost items

Input stack: `a b`

Output stack: `a b a b`

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items available on the stack

## Examples

```
0x10 0x20 2DUP => 0x10 0x20 0x10 0x20 
```

## Tests

```
0x10 0x20 2DUP => 0x10 0x20 0x10 0x20
```