# DEF

{% method -%}

Defines a instruction with a closure.

Input stack: `c w`

Output stack:

Since it is rather bothersome to keep repeating code over and over,
it'd be nice to be able define instructions as composites of other for the
scope of the program.

`DEF` allows to define instruction's program for the scope of the script's
remainder.

`DEF` will put the second topmost item off the stack (`c`) into the
instruction referenced by top item (`w`)

{% common -%}

```
PumpkinDB> [DUP DUP] 'dup2 DEF 1 dup2
1 1 1
```

{% endmethod %}

## Allocation

None.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack.

It will error if the format of the instruction is incorrect

It may error if this instruction is a built-in instruction that was previously
defined.

## Tests

```test
works : [DUP DUP] 'dup2 DEF 1 dup2 3 WRAP 1 1 1 3 WRAP EQUAL?.
empty_stack : [DEF] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ['a DEF] TRY UNWRAP 0x04 EQUAL?.
```
