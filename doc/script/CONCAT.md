# CONCAT

Takes two topmost items and concatenates them, pushes the
result to the top of the stack

Input stack: `a b`

Output stack: `ab`

## Allocation

Allocates for a result of concatenation

## Errors

EmptyStack error if there are less than two items on the stack

## Examples

```
"Hello, " "world!" => "Hello, world!"
```

## Tests

```
"Hello, " "world!" => "Hello, world!"
```