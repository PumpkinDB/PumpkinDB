# NONE?

Tests if the topmost stack item's length is equal to zero.

Input stack: `a`

Output stack: `c`

`NONE?` will push `1` if the item's length is equal to zero, `0` otherwise.

It has a "sister" word of [SOME?](SOMEP.md). Together they allow to
express a concept of an optional value.

## Allocation

None

## Errors

EmptyStack error if there is less than one items on the stack

## Examples

```
[] NONE? => 1
[1] NONE? => 0
```

## Tests

```
[] NONE? => 1
[1] NONE? => 0
```