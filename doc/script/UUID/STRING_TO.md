# UUID/STRING->

{% method -%}

Converts UUID string to a UUID binary

Input stack: `uuid string`

Output stack: `uuid binary`

{% common -%}

```
PumpkinDB> "59cef701-9fd8-4904-a18a-bcc0cb7e552e" UUID/STRING->.
0x59cef7019fd84904a18abcc0cb7e552e
```

{% endmethod %}

## Allocation

Allocates for parsing UUID and writing bytes.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one items on the stack

[InvalidValue](../errors/InvalidValue.md) if supplied string is not valid.


## Tests

```test
works : 0x59cef7019fd84904a18abcc0cb7e552e "59cef701-9fd8-4904-a18a-bcc0cb7e552e" UUID/STRING-> EQUAL?.
invalid_str : ["f00" UUID/STRING->] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [UUID/STRING->] TRY UNWRAP 0x04 EQUAL?.
```
