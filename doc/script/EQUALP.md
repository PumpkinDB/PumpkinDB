# EQUAL?

Compares two topmost items for equality.

Input stack: `a b`

Output stack: `c`

`EQUAL?` will push `1` if they are equal, `0` otherwise.

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

## Examples

```
"Hello, " "world!" EQUAL? => 0
```

## Tests

```
"Hello, " "world!" EQUAL? => 0
"Hello, " "Hello, " EQUAL? => 1
```