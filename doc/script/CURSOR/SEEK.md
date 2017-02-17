# CURSOR/NEXT

Sets the cursor at the key value pair with a greater or equal key

Input stack: `cursor key`

Output stack: `[key value]` or `[]`

If there is a key/value pair in the database that has a key
that is greater or equal to `key`, `[key value]` will be pushed onto the stack.
Otherwise, `[]` will be pushed. Useful in conjunction with [UNWRAP](../UNWRAP.md),
[SOME?](../SOMEP.md) and [NONE?](../NONEP.md).

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["3" "3" ASSOC COMMIT] WRITE [CURSOR 'c SET c "2" CURSOR/SEEK] READ UNWRAP => "3" "3"
```

## Tests

```
["3" "3" ASSOC COMMIT] WRITE [CURSOR 'c SET c "2" CURSOR/SEEK] READ UNWRAP => "3" "3"
```
