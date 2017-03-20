# [NAME]

{% method -%}

[One-line description]

Input stack: [stack notation]

Output stack: [stack notation]

[Multi-line description (if any necessary)]

{% common -%}

[Provide useful examples]

{% endmethod %}

## Allocation

[Describe heap and runtime allocation profile]

## Errors

[Describe in which cases this instruction will fail the program]

## Tests

```test
example_test : 1 1 EQUAL?.
```

---
#### Template notes

##### Stack notation

In this notation, the top of the stack is to the right.  Instructions
may also be shown in context when appropriate.

##### Filename transliteration

Some of the instructions contain characters not permitted or desirable
in the file system. Below is the table for transliterating them:

| Character | Replacement |
|-----------|-------------|
| ?         |  Q          |
| :         |  _          |
| !         |  B          |
| #         |  _          |
| $         |  _          |
| %         |  _          |
| @         |  _          |

As you can see, many of them resolve to `_`, which might create
collissions (but, frankly, one should avoid having something like
`Something$This` AND `Something%This`.
