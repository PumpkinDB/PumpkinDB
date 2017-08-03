# DOWHILE

{% method -%}

Evaluates code while there's `1` on top of the stack

Input stack: `code`

Output stack:

`DOWHILE` evaluates the closure on a new stack, popping the
current stack back after each evaluation.

{% common -%}

```
PumpkinDB> 1 >Q [Q> 1 UINT/ADD DUP >Q 4 EQUAL? NOT] DOWHILE Q> 
4
```

{% endmethod %}

## Allocation

Runtime allocation for code generation

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than one item on the stack

[InvalidValue](./errors/InvalidValue.md) error if the value being checked for truth is not a boolean.

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
works : 1 >Q [Q> 1 UINT/ADD DUP >Q 4 EQUAL? NOT] DOWHILE Q> 4 EQUAL?.
invalid_code : [1 DOWHILE] TRY UNWRAP 0x05 EQUAL?.
invalid_value : [[5] DOWHILE] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [DOWHILE] TRY UNWRAP 0x04 EQUAL?.
```
