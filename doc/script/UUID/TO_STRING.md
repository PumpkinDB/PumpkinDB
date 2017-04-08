# UUID/->STRING

{% method -%}

Converts UUID bytes to string representation

Input stack: `uuid bytes`

Output stack: `uuid str`

{% common -%}

```
PumpkinDB> 0x59cef7019fd84904a18abcc0cb7e552e UUID/->STRING.
"59cef701-9fd8-4904-a18a-bcc0cb7e552e"
```

{% endmethod %}

## Allocation

Allocates for string respresentation.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied string is not valid.


## Tests

```test
works : "59cef701-9fd8-4904-a18a-bcc0cb7e552e" 0x59cef7019fd84904a18abcc0cb7e552e UUID/->STRING EQUAL?.
invalid_str : [0x1 UUID/->STRING] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [UUID/->STRING] TRY UNWRAP 0x04 EQUAL?.
```
