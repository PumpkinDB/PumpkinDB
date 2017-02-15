# STACK

Takes the entire stack and pushes it back as a byte array

Input stack: `...`

Output stack: `a`

`STACK` takes the entire stack and pushes it as a binary
form PumpkinScript onto the stack. If passed to [UNWRAP](UNWRAP.md),
the same stack will be restored.

## Allocation

Allocates for the new values

## Errors

None

## Examples

```
1 2 3 STACK => 0x111213
1 2 3 STACK UNWRAP => 0x1 0x2 0x3
```

## Tests

```
1 2 3 STACK => 0x111213
1 2 3 STACK UNWRAP => 0x1 0x2 0x3
```