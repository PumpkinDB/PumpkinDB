# NOT

Negates a boolean value.

Input stack: `a`

Output stack: `c`

`NOT` will push `1` if `a` is `0` and `0` if `a` is `1`.

## Allocation

None

## Errors

InvalidValue error if the value being negated is not a boolean.

## Examples

```
0x01 NOT => 0
0x00 NOT => 1
```

## Tests

```
0x00 NOT => 1
0x01 NOT => 0
```