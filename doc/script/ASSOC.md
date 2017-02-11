# ASSOC

Takes the topmost item from the stack as a value and second
topmost item as a key and associates them in the database

Input stack: `key value`

Output stack:

This is the primary way of insert data into the database.
Only valid within [WRITE's](WRITE.md) scope. Can only be used
to insert new keys.

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

NoTransaction error if there's no current write transaction

## Examples

```
["hi" "there" ASSOC COMMIT] WRITE
```

## Tests

```
"hi" DUP "there" [ASSOC COMMIT] WRITE [RETR] READ => "there"
"hi" DUP "there" [ASSOC] WRITE [ASSOC?] READ => 0
```