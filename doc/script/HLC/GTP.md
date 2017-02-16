# HLC/GT?

Compares two topmost HLC items.

Input stack: `a b`

Output stack: `c`

`HLC/GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

## Allocation

None

## Errors

[EmptyStack](../ERRORS/EmptyStack.md) error if there are less than two items on the stack

It will fail if any of the top two items is not an HLC timestamp.

## Examples

```
HLC HLC SWAP HLC/GT? => 1
```

## Tests

```
HLC HLC SWAP HLC/GT? => 1
```
