# ASSOC?

{% method -%}

Takes the topmost item from the stack and pushes `1` to the stack
if the key is present, `0` if it is not.

Input stack: `key`

Output stack: `0` or `1`

This is the primary way of testing presence of a key in the database.
Only valid within [WRITE's](WRITE.md) or [READ's](READ.md) scopes.

{% common -%}

This example demonstrates testing of the key presence both within
READ and WRITE transaction scopes:

```
PumpkinDB> ["hi" "there" ASSOC "hi" ASSOC? COMMIT] WRITE ["hello" ASSOC?] READ
0x01 0x00
```

Because `hello` key was not associated (only `hi` was), the second boolean value
is `0x00` (false).

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[NoTransaction](./errors/NoTransaction.md) error if there's no current read or write transaction

## Tests

```test
present : "hi" DUP "there" [ASSOC COMMIT] WRITE [ASSOC?] READ.
not_present : "hi" "there" [ASSOC COMMIT] WRITE "bye" [ASSOC?] READ NOT.
assocp_requires_txn : [0 ASSOC?] TRY UNWRAP 0x08 EQUAL?.
assoc_requires_one_item : [[ASSOC] WRITE] TRY UNWRAP 0x04 EQUAL?.
```
