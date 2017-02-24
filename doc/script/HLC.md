# HLC

{% method -%}

Pushes Hybrid Logical Timestamp onto the stack

Input stack:

Output stack: `a`

Every timestamp is guaranteed to be unique and grow monotonically.

{% common -%}

```
PumpkinDB> HLC
0x000014A27859A0C2E2900000
```

{% endmethod %}

## Allocation

Allocates for the timestamp to be pushed on stack.

## Errors

None

## Tests

```test
inequality : HLC HLC EQUAL? NOT.
growth : HLC HLC LT?.
```
