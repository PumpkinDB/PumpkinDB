# OVER

Copies the second topmost item to the top of the stack

Input stack: `a b`

Output stack: `a b a`

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

## Examples

```
0x10 0x20 OVER => 0x10 0x20 0x10
```

## Tests

```
0x10 0x20 OVER => 0x10 0x20 0x10
```