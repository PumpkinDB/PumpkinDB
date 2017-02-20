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

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

[NoTransaction](./ERRORS/NoTransaction.md) error if there's no current write transaction

UnknownKey error if there is no such key. See [ASSOC?](ASSOCQ.md)
for mediating this problem

## Examples

```
"hi" [ASSOC?] READ
```

## Tests

```test
works : "hi" "there" 2DUP [ASSOC COMMIT] WRITE SWAP [RETR] READ EQUAL?.
requires_txn : ["hi" RETR] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [RETR] TRY UNWRAP 0x04 EQUAL?.
```