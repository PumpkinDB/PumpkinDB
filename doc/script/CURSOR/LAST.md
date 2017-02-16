# CURSOR/LAST

Sets the cursor at the last key value

Input stack: `cursor`

Output stack: `[key value]` or `[]`

If there is a last key/value pair in the database, `[key value]` will be pushed onto the stack.
Otherwise, `[]` will be pushed. Useful in conjunction with [UNWRAP](../UNWRAP.md),
[SOME?](../SOMEP.md) and [NONE?](../NONEP.md).

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST] READ UNWRAP => "1" "2"
```

## Tests

```
["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST] READ UNWRAP => "1" "2"
```
