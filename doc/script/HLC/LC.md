# HLC/LC

{% method -%}

Returns HLC timestamp's logical counter

Input stack: `a`

Output stack: `b`

Removes a topmost item off the stack (an HLC timestamp) and pushes
its logical counter as a 4-byte big-endian number.

{% common -%}

```
PumpkinDB> HLC DUP HLC/LC SWAP HLC/TICK DUP HLC/LC SWAP HLC/TICK HLC/LC
0x00000000 0x00000001 0x00000002
```

{% endmethod %}

## Allocation

Allocates for the logical counter to be pushed on stack.

## Errors

EmptyStack error if there are less than one item on the stack

It will fail if the item is not an HLC timestamp.

## Tests

```test
lc : HLC HLC/TICK HLC/LC 0x00000001 EQUAL?.
invalid_value : [1 HLC/LC] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [HLC/LC] TRY UNWRAP 0x04 EQUAL?.
```
