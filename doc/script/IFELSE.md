# IFELSE

## Usage

```
BOOL [THEN] [ELSE] IFELSE
```

Provides conditional flow control executing different branches of
code depending on a boolean value.

Input stack: `a [b] [c]`

Output stack: maybe `b`, maybe `c`

`IFELSE` will push the result of `[c]` to the stack if `a` is 0, or it
will push `[b]` otherwise.


## Allocation

None

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than three items on the stack

[InvalidValue](./ERRORS/InvalidValue.md) error if the value being checked for truth is not a boolean.

## Examples

```
0x01 [0x20] [0x30] IFELSE => 0x20
0x00 [0x20] [0x30] IFELSE => 0x30
```

## Tests

```
0x01 [0x20] [0x30] IFELSE => 0x20
0x00 [0x20] [0x30] IFELSE => 0x30
```