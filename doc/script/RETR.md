# RETR

Takes the topmost item from the stack as a key and looks
up its value in the database.

Input stack: `key`

Output stack: `value`

This is one of the ways to read data from the database.
Only valid within [WRITE's](WRITE.md) or [READ's](READ.md) scopes.
Can only be used to retrieve keys that were used.

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

NoTransaction error if there's no current write transaction

UnknownKey error if there is no such key. See [ASSOC?](ASSOCQ.md)
for mediating this problem

## Examples

```
"hi" [ASSOC?] READ
```

## Tests

```
"hi" DUP "there" [ASSOC COMMIT] WRITE [ASSOC?] READ => 1
"hi" DUP "there" [ASSOC] WRITE [ASSOC?] READ => 0
```