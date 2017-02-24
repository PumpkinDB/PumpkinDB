# COMMIT

{% method -%}

Commits current write transaction

Input stack: 

Output stack:

If not used, write transaction, once finished, will be discarded.
Only valid within [WRITE's](WRITE.md) scope. 

{% common -%}

```
PumpkinDB> ["hi" "there" ASSOC] WRITE ["hi" "there" ASSOC COMMIT] WRITE
```

In this example, the second transaction did not fail with a duplicate
key error because the first one never committed the change.

{% endmethod %}

## Allocation

None

## Errors

[NoTransaction](./errors/NoTransaction.md) error if there's no current write transaction


## Tests

```test
change : "hi" DUP "there" [ASSOC COMMIT] WRITE [ASSOC?] READ.
otherwise_no_change : "hi" DUP "there" [ASSOC] WRITE [ASSOC?] READ NOT.
commit_requires_txn : [COMMIT] TRY UNWRAP 0x08 EQUAL?.
commit_requires_write_txn : [[COMMIT] READ] TRY UNWRAP 0x08 EQUAL?.
```