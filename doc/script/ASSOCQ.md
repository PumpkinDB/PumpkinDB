# ASSOC?

Takes the topmost item from the stack and pushes `1` to the stack
if the key is present, `0` if it is not.

Takes the topmost item from the stack as a value and second
topmost item as a key and associates them in the database

Input stack: `key`

Output stack: `0` or `1`

This is the primary way of testing presence of a key in the database.
Only valid within [WRITE's](WRITE.md) or [READ's](READ.md) scopes.

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

NoTransaction error if there's no current read or write transaction

## Examples

```
["hi" "there" ASSOC COMMIT] WRITE
```

## Tests

```
"hi" DUP "there" [ASSOC COMMIT] WRITE [RETR] READ => "there"
"hi" DUP "there" [ASSOC] WRITE [ASSOC?] READ => 0
```