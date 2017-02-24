# HLC/TICK

{% method -%}

Increments a logical counter in an HLC timestamp

Input stack: `a`

Output stack: `b`

Removes a topmost item off the stack (an HLC timestamp) and increments
a logical counter, without updating the wall clock part.

{% common -%}

```
PumpkinDB> HLC DUP HLC/TICK
0x000014A278ED90AB13700000 0x000014A278ED90AB13700001
```

{% endmethod %}

## Allocation

Allocates for the new timestamp to be pushed on stack.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

It will fail if the item is not an HLC timestamp.

## Tests

```test
growth : HLC DUP HLC/TICK LT?.
invalid_value : [1 HLC/TICK] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [HLC/TICK] TRY UNWRAP 0x04 EQUAL?.
```
