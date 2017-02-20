# DUP

Duplicates an item at the top of the stack

Input stack: `a`

Output stack: `a a`

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if nothing is available on the stack

## Examples

```
0x10 DUP => 0x10 0x10 
```

## Tests

```test
works : 1 1 DUP EQUAL?.
empty_stack : [DUP] TRY UNWRAP 0x04 EQUAL?.
```