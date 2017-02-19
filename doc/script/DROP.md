# DROP

Drops an item off the top of the stack

Input stack: `a`

Output stack: 

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if nothing is available on the stack

## Examples

```
0x10 DROP =>
```

## Tests

```test
works : 1 DROP DEPTH 0 EQUAL?.
empty_stack : [DROP] TRY UNWRAP 0x04 EQUAL?.
```