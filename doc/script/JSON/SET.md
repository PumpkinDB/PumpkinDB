# JSON/GET

{% method -%}

Sets JSON value to a key in an object

Input stack: `json key value`

Output stack: `json`

{% common -%}

```
PumpkinDB> "{\"a\": 1}" "a" "2" JSON/SET
"{\"a\": 2}"
```

{% endmethod %}

## Allocation

Allocates for parsing and encoding JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than three items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied JSON is not valid.

[InvalidValue](../errors/InvalidValue.md) if supplied key is not a valid UTF-8 string.

[InvalidValue](../errors/InvalidValue.md) if supplied value is not a valid JSON.

## Tests

```test
works : "{\"a\": 1}" "a" "2" JSON/SET "a" JSON/GET "2" EQUAL?.
non_json : ["z" "a" "1" JSON/SET] TRY UNWRAP 0x03 EQUAL?.
invalid_key : ["{}" 0xffff "1" JSON/SET] TRY UNWRAP 0x03 EQUAL?.
invalid_val : ["{}" 0xffff "z" JSON/SET] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [JSON/SET] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ["1" JSON/SET] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : ["1" "2" JSON/SET] TRY UNWRAP 0x04 EQUAL?.
```
