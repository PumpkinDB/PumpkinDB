# UNWRAP

Takes the topmost item and evaluates it as a PumpkinScript
values-only program on the current stack

Input stack: `vals`

Output stack: all the values

UNWRAP is a "safe" cousin of [EVAL](EVAL.md). It takes a closure
and as long as it does not contain any words, evaluates it (essentially,
putting all the data on the stack)

It's particularly useful in conjunction with [NONE?](NONEP.md) and
[SOME?](SOMEP.md).

## Allocation

Runtime allocation during parsing

## Errors

EmptyStack error if there is less than one item on the stack

InvalidValue error if there are words in the item


## Examples

```
[1 2 3] UNWRAP => 1 2 3
```

## Tests

```
[1 2 3] UNWRAP => 1 2 3
[] UNWRAP =>
```