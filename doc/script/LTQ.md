# LT?

Compares two topmost items lexicographically.

Input stack: `a b`

Output stack: `a`

`LT?` will push `1` if `a` is strictly less than `b`, `0` otherwise.

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
0x10 0x20 LT? => 1
0x20 0x10 LT? => 0
```

## Tests

```test
less : 0x10 0x20 LT?.
greater : 0x20 0x10 LT? NOT.
equal : 0x10 0x10 LT? NOT.
requires_two_items_0 : [LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 LT?] TRY UNWRAP 0x04 EQUAL?.
```
