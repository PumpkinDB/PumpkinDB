# STACK

{% method -%}

Takes the entire stack and pushes it back as a byte array

Input stack: `...`

Output stack: `a`

`STACK` takes the entire stack and pushes it as a binary
form PumpkinScript onto the stack. If passed to [UNWRAP](UNWRAP.md),
the same stack will be restored.

{% common -%}

```
PumpkinDB> 1 2 3 STACK
0x111213
PumpkinDB> 1 2 3 STACK UNWRAP
0x1 0x2 0x3
```

{% endmethod %}

## Allocation

Allocates for the new values

## Errors

None

## Tests

```test
works : 1 2 3 STACK [1 2 3] EQUAL?.
work_nil : STACK [] EQUAL?.
```
