# UNWRAP

{% method -%}

Takes the topmost item and evaluates it as a PumpkinScript
values-only program on the current stack

Input stack: `vals`

Output stack: all the values

UNWRAP is a "safe" cousin of [EVAL](EVAL.md). It takes a closure
and as long as it does not contain any instructions, evaluates it (essentially,
putting all the data on the stack)

It's particularly useful in conjunction with [NONE?](NONEQ.md) and
[SOME?](SOMEQ.md).

{% common -%}

```
PumpkinDB> [1 2 3] UNWRAP
1 2 3
```

{% endmethod %}

## Allocation

Runtime allocation during parsing

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

[InvalidValue](./errors/InvalidValue.md) error if there are instructions in the item

## Tests

```test
works : [3 2 1 ] UNWRAP.
empty_stack : [UNWRAP] TRY UNWRAP 0x04 EQUAL?.
```
