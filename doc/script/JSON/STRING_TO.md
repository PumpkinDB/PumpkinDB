# JSON/STRING->

{% method -%}

Converts JSON string to a binary string

Input stack: `json`

Output stack: `str`

{% common -%}

```
PumpkinDB> "\"a\"" JSON/STRING->
"a"
```

{% endmethod %}

## Allocation

Allocates for parsing JSON and encoding a string

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied string is not valid.


## Tests

```test
works : "\"a\"" JSON/STRING-> "a" EQUAL?.
invalid_str : [0xffff JSON/STRING->] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [JSON/STRING->] TRY UNWRAP 0x04 EQUAL?.
```
