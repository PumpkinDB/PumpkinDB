# TXID

{% method -%}

Pushes current transaction's identifier onto the top of the stack 

Input stack: -

Output stack: `transaction id`

In certain cases it is worthwhile to learn whether some data was recorded
within the boundaries of the same transaction as other. This instruction allows
to do this by retrieving ongoing transaction's ID

Transaction identifiers are guaranteed to be unique and grow monotonically,
thus allowing one to compare the order of their origination.

Exact length is not guaranteed, however, for the purpose of comparability,
it is guaranteed that all transaction identifiers are of the same length.

{% common -%}

```
PumpkinDB> [ TXID ] READ.
0x0000000014cbed8aff34dc5000000000
```

{% endmethod %}

## Allocation

None

## Errors

[NoTransaction](./errors/NoTransaction.md) error if there's no ongoing transaction.

## Tests

```test
read : [TXID] READ SOME?.
write : [TXID] WRITE SOME?.
same_within_tx : [TXID TXID] WRITE EQUAL?.
ordering : [TXID] WRITE [TXID] WRITE LT?.
requires_txn : [TXID] TRY UNWRAP 0x08 EQUAL?.
```
