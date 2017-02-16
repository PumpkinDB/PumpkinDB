# CURSOR

Creates a read cursor

Input stack: 

Output stack: `cursor`

This is the primary way of navigating the database. This word
creates a cursor in a given transactional context and pushes its
identifier onto the stack.

Only valid within [WRITE's](WRITE.md) or [READ](READ.md) scope.

## Allocation

None

## Errors

NoTransaction error if there's no current write transaction

## Examples

```
[CURSOR 'c SET ...] READ
```

## Tests

