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

```
1 2 3 [1 EQUAL? NOT] DOWHILE =>
```