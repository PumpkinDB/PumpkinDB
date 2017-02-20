# CONCAT

Takes two topmost items and concatenates them, pushes the
result to the top of the stack

Input stack: `a b`

Output stack: `ab`

## Allocation

Allocates for a result of concatenation

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

## Examples

```
"Hello, " "world!" => "Hello, world!"
```

## Tests

```test
concat : "Hello, " "world!" CONCAT "Hello, world!" EQUAL?.
concat_requires_two_items_0 : [CONCAT] TRY UNWRAP 0x04 EQUAL?.
concat_requires_two_items_1 : [1 CONCAT] TRY UNWRAP 0x04 EQUAL?.
```