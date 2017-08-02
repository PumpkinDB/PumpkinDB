# IFELSE

{% method -%}

Provides conditional flow control executing different branches of
code depending on a boolean value.

Input stack: `a [b] [c]`

Output stack: maybe `b`, maybe `c`

`IFELSE` will push the result of `[c]` to the stack if `a` is 0, or it
will push `[b]` otherwise.

`IFELSE` evaluates the branches closures on the current stack.
{% common -%}

```
PumpkinDB> 0x01 [0x20] [0x30] IFELSE
0x20
PumpkinDB> 0x00 [0x20] [0x30] IFELSE
0x30
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items on the stack

[InvalidValue](./errors/InvalidValue.md) error if the value being checked for truth is not a boolean.

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
works : 1 [2] [3] IFELSE 2 EQUAL?.
works_else : 0 [2] [3] IFELSE 3 EQUAL?.
invalid_code : [1 1 [] IFELSE] TRY UNWRAP 0x05 EQUAL?.
invalid_code_1 : [0 [] 1 IFELSE] TRY UNWRAP 0x05 EQUAL?.
invalid_value : [5 [1] [2] IFELSE] TRY UNWRAP 0x03 EQUAL?.
requires_three_items_0 : [IFELSE] TRY UNWRAP 0x04 EQUAL?.
requires_three_items_1 : [[] IFELSE] TRY UNWRAP 0x04 EQUAL?.
requires_three_items_1 : [[] [] IFELSE] TRY UNWRAP 0x04 EQUAL?.
```
