# CURSOR/FIRST?

Sets the cursor at the first key value

Input stack: `cursor`

Output stack: `b`

If there is a first key/value pair in the database, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

Useful in conjunction with [CURSOR/CUR](CUR.md)

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["1" "2" ASSOC COMMIT] WRITE [  CURSOR 'c SET c CURSOR/FIRST?] READ => 1
```

## Tests

```
["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST?] READ  => 1
```
