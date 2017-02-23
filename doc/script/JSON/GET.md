# JSON/GET

{% method -%}

Retrieves JSON value by key from an object

Input stack: `json key`

Output stack: `b`

{% common -%}

```
PumpkinDB> "{\"a\": 1}" "a" JSON/GET
0x01
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied JSON is not valid.

[InvalidValue](../errors/InvalidValue.md) if supplied key is not a valid UTF-8 string.

[InvalidValue](../errors/InvalidValue.md) if supplied key is present.

## Tests

```test
works : "{\"a\": 1}" "a" JSON/GET "1" EQUAL?.
non_json : ["z" "a" JSON/GET] TRY UNWRAP 0x03 EQUAL?.
invalid_key : ["{}" 0xffff JSON/GET] TRY UNWRAP 0x03 EQUAL?.
non_present_key : ["{}" "a" JSON/GET] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [JSON/GET] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ["1" JSON/GET] TRY UNWRAP 0x04 EQUAL?.
```
