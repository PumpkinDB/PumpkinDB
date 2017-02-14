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

InvalidValue error if the value being checked for truth is not a boolean.

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