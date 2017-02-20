# HLC

Pushes Hybrid Logical Timestamp onto the stack

Input stack:

Output stack: `a`

Every timestamp is guaranteed to be unique and grow monotonically. 

## Allocation

Allocates for the timestamp to be pushed on stack.

## Errors

None

## Examples

```
HLC => 0x000014A27859A0C2E2900000
```

## Tests

```test
inequality : HLC HLC EQUAL? NOT.
growth : HLC HLC HLC/LT?.
```