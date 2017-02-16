# GT?

Compares two topmost items lexicographically.

Input stack: `a b`

Output stack: `a`

`GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
0x10 0x20 GT? => 0
0x20 0x10 GT? => 1
```

## Tests

```
0x10 0x20 GT? => 0
0x20 0x10 GT? => 1
0x10 0x10 GT? => 0
```
