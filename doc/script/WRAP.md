# WRAP

Takes a portion of the stack and pushes it back as a byte array

Input stack: `... n`

Output stack: `a`

`WRAP` takes `n` items off the top of the stack and pushes them as a binary
form PumpkinScript onto the stack. If passed to [UNWRAP](UNWRAP.md),
the same stack portion will be restored.

## Allocation

Allocates for the new values

## Errors

None

## Examples

```
1 2 3 2 WRAP => 0x1 0x1213
1 2 3 2 WRAP UNWRAP => 0x1 0x2 0x3
```

## Tests

```
1 2 3 2 WRAP => 0x1 0x1213
1 2 3 2 WRAP UNWRAP => 0x1 0x2 0x3
```