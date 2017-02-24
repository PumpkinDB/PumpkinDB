# JSON/EMPTY

{% method -%}

Returns an empty JSON object

Input stack: ``

Output stack: "{}"

{% common -%}

```
PumpkinDB> JSON/EMPTY
"{}"
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : JSON/EMPTY "{}" EQUAL?.
```
