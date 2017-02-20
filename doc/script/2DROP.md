# 2DROP

Drops two topmost items off the top of the stack

Input stack: `a b`

Output stack: 

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items available on the stack

## Examples

```
0x10 0x20 2DROP =>
```

## Tests

```test
2drop_drops_two_items : 1 2 2DROP STACK LENGTH 0 EQUAL?.
2drop_requires_two_items_0 : [2DROP] TRY UNWRAP 0x04 EQUAL?.
2drop_requires_two_items_1 : [1 2DROP] TRY UNWRAP 0x04 EQUAL?.
```