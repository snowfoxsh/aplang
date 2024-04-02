# Statements
### If

```ap
IF (condition) {
	// body
}

IF (condition) {
	// body
} ELSE IF (condition) {
	// body
} ELSE {
    // body
} 
```

```rust
struct If {
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
struct RepeatTimes {
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
struct RepeatUntil {
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
struct ForEach {
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
struct Procedure {
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
    DISPLAY(message)
} else {
    DISPLAY("It is not 4")
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

```
program -> 
```