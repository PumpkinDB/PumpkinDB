# HASH/SHA1

{% method -%}

Puts the SHA-1 hash of the top item on the stack back to the top of the stack

Input stack: `a`

Output stack: `b`

Please note that a SHA-1 collision technique [has been found](https://shattered.io/).
Consider using other hashes like [SHA256](SHA256.md).

{% common -%}

```
PumpkinDB> "The quick brown fox jumps over the lazy dog" HASH/SHA1
0x2fd4e1c67a2d28fced849ee1bb76e7391b93eb12
```

{% endmethod %}

## Allocation

Allocates for the result of the hashing

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are no items on the stack

## Tests

```test
works : "The quick brown fox jumps over the lazy dog" HASH/SHA1 0x2fd4e1c67a2d28fced849ee1bb76e7391b93eb12 EQUAL?.
empty_stack : [HASH/SHA1] TRY UNWRAP 0x04 EQUAL?.
```
