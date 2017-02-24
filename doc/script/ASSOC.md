# ASSOC

{% method -%}

Takes the topmost item from the stack as a value and second
topmost item as a key and associates them in the database

Input stack: `key value`

Output stack:

This is the primary way of insert data into the database.
Only valid within [WRITE's](WRITE.md) scope. Can only be used
to insert new keys.

{% common -%}

This associates key `hi` and value `there` in the database
and commits the changes:

```
PumpkinDB> ["hi" "there" ASSOC COMMIT] WRITE
```

{% endmethod %}

## Allocation

None

## Errors

EmptyStack error if there are less than two items on the stack

NoTransaction error if there's no current write transaction

## Tests

```test
assoc_commit : 0 1 2DUP [ASSOC COMMIT] WRITE SWAP [RETR] READ EQUAL?.
assoc_no_commit : 0 DUP 1 [ASSOC] WRITE [ASSOC?] READ NOT.
assoc_requires_two_items_0 : [[ASSOC] WRITE] TRY UNWRAP 0x04 EQUAL?.
assoc_requires_two_items_1 : [[0 ASSOC] WRITE] TRY UNWRAP 0x04 EQUAL?.
assoc_requires_txn : [ASSOC] TRY UNWRAP 0x08 EQUAL?.
assoc_requires_write_txn : [[ASSOC] READ] TRY UNWRAP 0x08 EQUAL?.
assoc_unique_key : 0 0 2DUP [ASSOC COMMIT] WRITE [[ASSOC] WRITE] TRY UNWRAP 0x06 EQUAL?. 
```