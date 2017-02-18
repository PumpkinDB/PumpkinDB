# IF

## Usage

```
BOOL [THEN] IF
```

Provides conditional flow control depending on a boolean value.

Input stack: `a [b]`

Output stack: maybe `b`

`IF` will push the result `[c]` to the stack if `a` is `0`.


## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](./ERRORS/InvalidValue.md) error if the value being checked for truth is not a boolean.

[Decoding error](./ERRORS/DECODING.md) error if the code is undecodable.

## Examples

```
0x01 [0x20] IF => 0x20
0x00 [0x20] IF =>
```

## Tests

```
0x01 [0x20] IF => 0x20
0x00 [0x20] IF =>
```