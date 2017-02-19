# EQUAL?

Compares two topmost items for equality.

Input stack: `a b`

Output stack: `c`

`EQUAL?` will push `1` if they are equal, `0` otherwise.

## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
"Hello, " "world!" EQUAL? => 0
```

## Tests

```test
not_equal : "Hello, " "world!" EQUAL? NOT.
equal : "Hello, " "Hello, " EQUAL?.
requires_two_items_0 : [EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 EQUAL?] TRY UNWRAP 0x04 EQUAL?.
```