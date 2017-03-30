# HLC/OBSERVE

{% method -%}

Updates HLC timestamp from provided value

Input stack: `a`

Output stack: `b`

Removes topmost item off the stack (an HLC timestamp) and updates Hybrid Logical
Clock based on its value.

{% common -%}

```
PumpkinDB> HLC HLC/OBSERVE.
0x0000000034b07f85d24f7dc800000000
```

{% endmethod %}

## Allocation

Allocates for the new timestamp to be pushed on stack.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are fewer than one item on the stack
[InvalidValue](./errors/InvalidValue.md) error if the given value is not a valid HLC.

## Tests

```test
observe_tick : HLC DUP HLC/TICK HLC/OBSERVE LT?.
invalid_value : [1 HLC/OBSERVE] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [HLC/OBSERVE] TRY UNWRAP 0x04 EQUAL?.
```
