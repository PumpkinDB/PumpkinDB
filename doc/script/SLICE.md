# SLICE

Pushes a subset of a byte array onto the stack

Input stack: `data start end`

Output stack: `new_data`

SLICE pushes a subset from include `start` to exclusive `end`
to the top of the stack.

## Allocation

Allocated in runtime to parse start/end numbers. Sliced
array is zero-copy.

## Errors

EmptyStack error if there are less than three items on the stack

InvalidValue error if `start` is larger than data length.

InvalidValue error if `start` is lesser than `end`.

InvalidValue error if `end` is larger than data length.
 

## Examples

```
0x102030 1 3 SLICE => 0x2030
```

## Tests

```
0x102030 1 3 SLICE => 0x2030
```
