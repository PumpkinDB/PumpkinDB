# EVAL/VALID?

{% method -%}

Assesses validity of the code from the perspective of decoding

Input stack: `code`

Output stack: `1` or `0`

`EVAL/VALID?` will only verify if PumpkinDB will be able to
interpret the code. However, it won't assess any other properties
pertaining to its validity.

Generally speaking, this instruction is only reserved for
some special cases as `EVAL` will fail upon trying to
evaluate incorrect code anyway.

{% common -%}

```
PumpkinDB> 1 EVAL/VALID?
0x00
PumpkinDB> 'DUP EVAL/VALID?
0x01
PumpkinDB> [1 DUP] EVAL/VALID?
0x01
```

{% endmethod %}

## Allocation

Allocates for parsing the binary representation of the program.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than
one item on the stack

## Tests

```test
positive : [1] EVAL/VALID?.
negative : 1 EVAL/VALID? NOT.
empty_stack : [EVAL/VALID?] TRY UNWRAP 0x04 EQUAL?.
```
