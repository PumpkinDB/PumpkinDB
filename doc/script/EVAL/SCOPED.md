# EVAL/SCOPED

## Graduated feature: `scoped_dictionary`

Takes the topmost item and evaluates it as a PumpkinScript
program on the current stack with a clone of the dictionary

Input stack: `code`

Output stack: result of `code` evaluation

`EVAL/SCOPED` is a sister version of [EVAL](../EVAL.md) with
one important distinction: all new words defined inside 
(or updated values for previously defined ones) within this
evaluation (using [SET](../SET.md) and [DEF](../DEF.md)) will be
gone after the evaluation.  
 
## Allocation

Allocates a copy of the code (this might change in the future)
during the runtime.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than one item on the stack

## Examples

```
1 'val SET [2 'val SET val] EVAL/SCOPED val => 2 1
```

## Tests

```test
eval_scoped_clone : "scoped_dictionary" FEATURE? [1 'val SET [2 'val SET val] EVAL/SCOPED val 2 WRAP 2 1 2 WRAP EQUAL?] [1] IFELSE.
```