# HLC/LC

Returns HLC timestamp's logical counter 

Input stack: `a`

Output stack: `b`

Removes a topmost item off the stack (an HLC timestamp) and pushes
its logical counter as a 4-byte big-endian number. 

## Allocation

Allocates for the logical counter to be pushed on stack.

## Errors

EmptyStack error if there are less than one item on the stack

It will fail if the item is not an HLC timestamp.


## Examples

```
HLC DUP HLC/TICK => 0x000014A278ED90AB13700000 0x000014A278ED90AB13700001
```

## Tests

```
HLC DUP HLC/TICK HLC/LT? => 1
```