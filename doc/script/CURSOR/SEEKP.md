# CURSOR/SEEK?

Sets the cursor at the key value pair with a greater or equal key

Input stack: `key cursor`

Output stack: `b`

If there is a key/value pair in the database that has a key
that is greater or equal to `key`, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

Useful in conjunction with [CURSOR/CUR](CUR.md)

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["3" "3" ASSOC COMMIT] WRITE [CURSOR 'c SET c "2" CURSOR/SEEK?] READ => 1
```

## Tests

```
["3" "3" ASSOC COMMIT] WRITE [CURSOR 'c SET c "2" CURSOR/SEEK?] READ => 1
```
