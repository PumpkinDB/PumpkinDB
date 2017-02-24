# JSON/->STRING

{% method -%}

Converts a binary string to a JSON string

Input stack: `str`

Output stack: `json`

{% common -%}

```
PumpkinDB> "a" JSON/->STRING
"\"a\""
```

{% endmethod %}

## Allocation

Allocates for parsing JSON and encoding a string

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied string is not a valid UTF-8 string.


## Tests

```test
works : "a" JSON/->STRING "\"a\"" EQUAL?.
invalid_str : [0xffff JSON/->STRING] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [JSON/->STRING] TRY UNWRAP 0x04 EQUAL?.
```
