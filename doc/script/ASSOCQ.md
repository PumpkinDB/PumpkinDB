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

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

[NoTransaction](./ERRORS/NoTransaction.md) error if there's no current read or write transaction

## Examples

```
["hi" "there" ASSOC COMMIT] WRITE
```

## Tests

```test
present : "hi" DUP "there" [ASSOC COMMIT] WRITE [ASSOC?] READ.
not_present : "hi" "there" [ASSOC COMMIT] WRITE "bye" [ASSOC?] READ NOT.
assocp_requires_txn : [0 ASSOC?] TRY UNWRAP 0x08 EQUAL?.
assoc_requires_one_item : [[ASSOC] WRITE] TRY UNWRAP 0x04 EQUAL?.
```