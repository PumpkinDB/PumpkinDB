# JSON/HAS?

{% method -%}

Tests JSON if it contains a key value pair with a given key

Input stack: `json key`

Output stack: `b`

`JSON/HAS?` will push `1` if supplied JSON has a key `key`, `0` otherwise

{% common -%}

```
PumpkinDB> "{\"a\": 1}" "a" JSON/HAS?
0x01
PumpkinDB> "{\"b\": 1}" "a" JSON/HAS?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied JSON is not valid.

[InvalidValue](../errors/InvalidValue.md) if supplied key is not a valid UTF-8 string.

## Tests

```test
works : "{\"a\": 1}" "a" JSON/HAS?.
works_negative : "{\"a\": 1}" "b" JSON/HAS? NOT.
non_json : ["z" "a" JSON/HAS?] TRY UNWRAP 0x03 EQUAL?.
invalid_key : ["{}" 0xffff JSON/HAS?] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [JSON/HAS?] TRY UNWRAP 0x04 EQUAL?.
```
