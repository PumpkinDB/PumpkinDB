# CURSOR/CUR?

Sets the cursor at the current key value

Input stack: `cursor`

Output stack: `b`

If there is a current key/value pair set in the cursor, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["1" "2" ASSOC COMMIT] WRITE [[c = CURSOR] SET c CURSOR/FIRST DROP c CURSOR/CUR?] READ UNWRAP => 1
```

## Tests

```
["1" "2" ASSOC COMMIT] WRITE [[c = CURSOR] SET c CURSOR/FIRST DROP c CURSOR/CUR?] READ UNWRAP => 1
```
