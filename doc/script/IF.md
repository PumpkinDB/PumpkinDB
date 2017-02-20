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

```test
works : 1 [2] IF 2 EQUAL?.
invalid_code : [1 1 IF] TRY UNWRAP 0x05 EQUAL?.
invalid_value : [5 [1] IF] TRY UNWRAP 0x03 EQUAL?.
requires_two_items_0 : [IF] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [[] IF] TRY UNWRAP 0x04 EQUAL?.
```