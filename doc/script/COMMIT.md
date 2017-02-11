# COMMIT

Commits current write transaction

Input stack: 

Output stack:

If not used, write transaction, once finished, will be discarded.
Only valid within [WRITE's](WRITE.md) scope. 

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

NoTransaction error if there's no current write transaction

DuplicateKey error if the key has been already used.

## Examples

```
["hi" "there" ASSOC COMMIT] WRITE
```

## Tests

```
"hi" DUP "there" [ASSOC COMMIT] WRITE [RETR] READ => "there"
```