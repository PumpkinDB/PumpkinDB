# DOWHILE

Evaluates code while there's `1` on top of the stack 

Input stack: `code`

Output stack: 

## Allocation

Runtime allocation for code generation

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than one item on the stack

[InvalidValue](./ERRORS/InvalidValue.md) error if the value being checked for truth is not a boolean.

[Decoding error](./ERRORS/DECODING.md) error if the code is undecodable.

## Examples

```
1 2 3 [1 EQUAL? NOT] DOWHILE =>
```

## Tests

```test
works : 1 2 3 [1 EQUAL? NOT] DOWHILE DEPTH 0 EQUAL?.
invalid_code : [1 DOWHILE] TRY UNWRAP 0x05 EQUAL?.
invalid_value : [[5] DOWHILE] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [DOWHILE] TRY UNWRAP 0x04 EQUAL?.
```