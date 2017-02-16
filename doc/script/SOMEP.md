# SOME?

Tests if the topmost stack item's length is larger than zero.

Input stack: `a`

Output stack: `c`

`SOME?` will push `1` if the item's length is larger than zero, `0` otherwise.

It has a "sister" word of [NONE?](NONEP.md). Together they allow to
express a concept of an optional value.

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than one items on the stack

## Examples

```
[] SOME? => 0
[1] SOME? => 1
```

## Tests

```
[] SOME? => 0
[1] SOME? => 1
```