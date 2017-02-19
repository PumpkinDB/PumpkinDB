# HLC/LT?

Compares two topmost HLC items.

Input stack: `a b`

Output stack: `c`

`HLC/LT?` will push `1` if `a` is strictly lesser than `b`, `0` otherwise.

## Allocation

None

## Errors

[EmptyStack](../ERRORS/EmptyStack.md) error if there are less than two items on the stack

It will fail if any of the top two items is not an HLC timestamp.

## Examples

```
HLC HLC HLC/LT? => 1
```

## Tests

```test
equal :  HLC DUP HLC/LT? NOT.
lesser : HLC DUP HLC/TICK HLC/LT?.
invalid_value : [1 HLC HLC/LT?] TRY UNWRAP 0x03 EQUAL?.
invalid_value_1 : [HLC 1 HLC/LT?] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [HLC/LT?] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [HLC HLC/LT?] TRY UNWRAP 0x04 EQUAL?.
```