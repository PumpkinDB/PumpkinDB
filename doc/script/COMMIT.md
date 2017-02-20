# COMMIT

Commits current write transaction

Input stack: 

Output stack:

If not used, write transaction, once finished, will be discarded.
Only valid within [WRITE's](WRITE.md) scope. 

## Allocation

None

## Errors

[NoTransaction](./ERRORS/NoTransaction.md) error if there's no current write transaction


## Examples

```
["hi" "there" ASSOC COMMIT] WRITE
```

## Tests

```test
change : "hi" DUP "there" [ASSOC COMMIT] WRITE [ASSOC?] READ.
otherwise_no_change : "hi" DUP "there" [ASSOC] WRITE [ASSOC?] READ NOT.
commit_requires_txn : [COMMIT] TRY UNWRAP 0x08 EQUAL?.
commit_requires_write_txn : [[COMMIT] READ] TRY UNWRAP 0x08 EQUAL?.
```