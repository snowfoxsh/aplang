# Statements
### If

```ap
IF (condition) {
	// body
}

IF (condition) {
	// body
} ELSE {
	// body
}
```

```rust
If {
	condition: Vec<Expr>,
	body: Vec<Statment>,
	alternate: Option<Vec<Statement>>,
} 
```

### Repeat times

```ap
REPEAT n TIMES {
	// body
}
```

```rust
RepeatTimes {
	count: Expr,
	body: Vec<Statement>,
}
```

Only accepts a numeric value. Will cast to an integer because everything is floats
### Repeat Until

```ap
REPEAT UNTIL(condition) {
	// body
}
```

```rust
RepeatUntil {
	condition: Expr,
	body: Vec<Statment>
}
```

### For

```ap
FOR EACH item IN list {
	// body
}
```

```rust
ForEach {
	item: Ident,
	list: Expr,
	body: Vec<Statement>
}
```

Only accepts an array or list as input

#### Procedure

```ap
PROCEDURE procName(param1, param2) {
	// body
}
```

```rust
Procedure {
	name: Ident,
	params: Vec<Ident>,
	body: Vec<Statement>
}
```

Can have the `RETURN` keyword in body


```ap
x <- 3

PROCEDURE addOne(value) {
    valPlusOne <- value + 1
    RETURN value
}

IF addOne(4) == x {
    message <- "it is 4"
    PRINT(message)
} else {
    PRINT("It is not 4")
}

list <- [1, 2, 3]

FOR EACH value IN list {
    PRINT(value)
}

i <- 0
REPEAT 3 TIMES {
    i <- i + 1
}

```
