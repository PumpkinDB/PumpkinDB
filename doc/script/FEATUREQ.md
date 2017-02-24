# FEATURE?

{% method -%}

Checks if a feature is enabled.

Input stack: `name`

Output stack: `b`

PumpkinDB is new and many (experimental) [features](../FEATURES.md) are being hashed out. Instead of
these features sitting in their own branches or Pull Requests, we decided to encourage
broader experimentation.

`FEATURE?` will push `1` if feature `name` is enabled, `0` otherwise.

{% common -%}

```
PumpkinDB> "scoped_dictionary" FEATURE?
1
PumpkinDB> "unknown_feature" FEATURE?
0
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

## Tests

```test
empty_stack : [FEATURE?] TRY UNWRAP 0x04 EQUAL?.
```
