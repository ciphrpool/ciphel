# ciphel

Ciphel is the programming language designed for CipherPool.

```ciphel

fn main() -> Unit {
	print("Hello, World");
}

main()
```

### Scopes

A scope in Ciphel is a delimited section of the program with the following characteristics:

- **Parent Scope**: Every scope, except the general scope, has a parent scope. The general scope acts as the top-level context.
- **Instructions and Variables**: Each scope contains a set of instructions and variables. Variables are identified by unique IDs.
- **Shadowing**: Variables within a scope can shadow variables from the parent scope. However, the original data in the parent scope remains unchanged after the inner scope concludes.
- **Stack and Heap Management**: Data on the stack created within a scope is cleared upon scope termination, excluding any return values. The heap remains unaffected by the scope's end.
- **Variable Resolution**: Variable resolution is hierarchical. If a variable is not found in the current scope, the search moves to the parent scope, continuing upward until the variable is located.
- **Variable Mutation**: Mutations to a variable are not confined to the scope in which the mutation occurs. Changes in a child scope are visible in its parent scope.
- **Function Handling**: Functions can be called like variables. Redefining a function in an inner scope only affects that specific scope.

#### General Scope

The general scope is the outermost layer of a Ciphel program, encompassing all other scopes and instructions. It serves as the primary execution context for the program.

##### Instruction Mutation

Within the general scope, instructions can undergo dynamic mutations. These mutations include:

- **Commit**: Appending a new instruction to the general scope.
- **Revert**: Deleting an existing instruction from the general scope.

### Events

Events in Ciphel are specialized functions that trigger under specific conditions. They possess the following properties:

- **Scope Independence**: Events are defined independently of the scope in which they were created.
- **Event Scope**: Each event has its own scope, with the general scope always acting as its parent. This allows events to access heap variables or variables defined in the general scope exclusively.

#### Event Triggers

The conditions for triggering an event are defined by a function that evaluates observables. This function takes as input one or more observables and returns a boolean value. An event is triggered when the condition function returns `true`.

##### Observables

Observables in Ciphel are elements whose state changes can trigger events. Each observable is monitored, and the condition function is executed whenever a mutation occurs. The following is a list of possible observables:

1. **Heap Content Change**: Triggered when the content at a given address in the heap changes.
2. **Cursor Movement**: Activated when a specified cursor position changes.
3. **Cell Mode Change**: Triggered when the mode of a given cell changes.
4. **Cell State Change**: Occurs when the state of a given cell changes.
5. **Cell Substate Change**: Triggered when the substate of a given cell changes.
6. **Cell Content Change**: Activated when the content of a given cell changes.
7. **Event Trigger**: Triggered when a specified event occurs.
8. **Command Execution**: Occurs when any command is performed.
9. **Command Failure**: Triggered when a command fails.
10. **Command Success**: Activated when a command is successfully executed.
11. **Specific Command Execution**: Triggered by the performance of a specified command.
12. **Energy Level Check**: Occurs when the given energy level is below or exceeds a specified value.
13. **ECR Level Check**: Triggered when the given ECR ( Energy Conservation Ratio ) level is below or exceeds a specified value.
14. **Instruction Commitment**: Activated when an instruction is committed to the program.
15. **Instruction Reversion**: Triggered when an instruction is reverted from the program.

### Instructions

#### Variable Declaration

- **Purpose**: Declares a new variable.
- **Syntax**: A variable is declared with an ID, a type, and an optional initial value.
- **Scope Registration**: The declared variable is registered in the current scope, potentially shadowing any existing variable with the same name.
- **Accessibility**: Once declared, the variable can be accessed and modified in all sub-scopes and the current scope.

#### Variable Assignment

- **Purpose**: Assigns a value to an existing variable.
- **Requirements**: The variable must exist in the current or an accessible parent scope.

#### Loop Constructs

Ciphel supports three types of loops:

1. **For Loop**: Iterates over an iterable (such as a slice or vector).
2. **While Loop**: Continues as long as a given condition evaluates to true.
3. **Infinite Loop**: Continuously executes without a terminating condition.

- **Loop Scope**: Each loop has an attached scope that is executed while the loop condition is met.

#### Function Definition

- **Purpose**: Defines a function.
- **Components**: A function is defined with an ID, parameter types, a return type (Unit for no return), and an attached scope.
- **Execution**: When called, the function executes its attached scope.

#### Event Definition

- **Purpose**: Defines an event.
- **Components**: An event is defined with an ID, a trigger condition, and an attached scope.

#### If Statement

- **Functionality**: Executes the main scope if the condition is true; otherwise, executes an optional else scope.

#### Match Statement

- **Purpose**: Pattern matching based on a given value.
- **Functionality**: Attempts to match the value with provided patterns. Pattern matching may introduce new variables accessible within the attached scope. If no pattern matches, an optional else scope is executed.

#### Try Statement

- **Purpose**: Error handling.
- **Functionality**: Executes an attached scope. If an error is raised, the error is discarded, and an optional else scope is executed.

#### Function Call

- **Purpose**: Calls a defined function.
- **Requirements**: The function must exist, and the call includes an ID and appropriate arguments.

#### Scope Creation

- **Purpose**: Creates a new scope.
- **Parent Scope**: The new scope has the current scope as its parent.

#### Return Statement

- **Purpose**: Ends the execution of the current scope and returns a value.

### CasmProgram

A casm (abbreviated as "sp") represents the minimal executable step of an instruction, with each casm having an associated weight that determines its execution cost. This weight is crucial in calculating the energy required for a player to perform an instruction.

#### Asm Calculation Rules

The calculation of casm weights follows specific rules for expressions and statements:

#### For Expressions:

1. **Static Data Creation**: 1sp.
2. **Data Access via Pointer Address**: 2sp.
3. **Addressing Data**: 1sp.
4. **Negating an Expression**: 1sp.
5. **Binary Operation**: Sum of the casm weights of the two expressions.
6. **Parentheses**: No additional casm weight for the inner expression.
7. **If Expression**: Sum of the casm weight of the condition and the maximum of the casm weights of the main and else expressions.
8. **Match Expression**: Asm weight of the given expression plus the maximum casm weight of the pattern expressions.
9. **Try Expression**: double the casm weight of the main expression plus the casm weight of the main and else expressions.
10. **Function Call**: 10% of the casm weight of the function plus the sum of the casm weights of the arguments.
    Platform api function call result in 10sp

#### For Statements:

1. **Declaration**: 2sp for pattern matching, 1sp otherwise, plus the casm weight of the value.
2. **Assignment**: 1sp for simple access, 2sp for accessing fields inside a struct or dereferencing an address.
3. **Loop**: Asm weight of the iterator or condition plus the casm weight of the scope every times the loop iterates, DYNAMIC sp.
4. **Function Definition**: 1sp per parameter if the parameter lives on the stack or is an address, plus the casm weight of the scope.
5. **Event Definition**: Asm weight of the condition plus casm weight of the scope.
   The casm weight of the condition depends on the observable involved in the trigger function, the more an observable is broad an the more the casm weight will be high, exemple the casm weight of the observable a command has been performed is higher than the command X has been performed
6. **If Statement**: Sum of the casm weight of the condition and the maximum of the casm weights of the main and else scopes.
7. **Match Statement**: Asm weight of the given expression and the maximum casm weight of the pattern scopes.
8. **Try Statement**: double the casm weight of the main scope plus the casm weights of the main and else scopes.
9. **Function Call**: 10% of the casm weight of the function plus the sum of the casm weights of the arguments.
10. **Scope**: Sum of the casm weights of the internal instructions.

### Error Handling

Error are value that if not properly handled propagate themselves and from variable to variable, corrupt heap and cells in the ribbon. An error is defined with a message and an energy cost. Any time an error is propagated the affected player receive the penalty, the energy cost of the error. The propation of an error can only be stopped by using try else blocks or freeing a corrupted variable before. An error can propagate itself in several ways :

- Affectation : when error is affected to a variable, the value of the variable is the error as long as the variable lives.
- If statement : if the condition of an if statement resolve to an error the chosen branch become non-deterministic and the variable that are modified inside the chosen scope receive the propagated error
- Match statement : if the expression that is being matched resolve to an error the chosen branch become non-deterministic and the variable that are modified inside the chosen scope receive the propagated error
- For loop : if the iterator yields an error the error propagate through the loop's attached scope with the iterator's item
- While loop : if the condition resolve to an error, the variable affected ( created , or mutated ) in the scope can be affected non deterministically by the error
- Call a function with an argument that is an error will lead to the propagation of the error inside the function scope, , the variable affected ( created , or mutated ) with the error arguments in the scope will get affected by the error
- return statement propagate error through the outer scope
- In a expression :
  - binary operation of an error result in an error
  - unary operation of an error result in an error
  - If expression : if the condition is an error the result is an error
  - match expression : if the value that is pattern match is an error the result is an error
- Error can be propagated in channel

### Error Handling

Error handling in Ciphel is a critical aspect of the game's strategy, as errors can propagate through variables, corrupting the heap and affecting cells on the ribbon. Proper management of errors is essential to maintain control over the game's flow and prevent penalties.

#### Error Definition

An error in Ciphel is defined by a message and an energy cost. When an error propagates, the affected player incurs the energy cost associated with that error.

#### Stopping Error Propagation

- **Try-Else Blocks**: The propagation of an error can be halted using try-else blocks.
- **Freeing Variables**: Freeing a corrupted variable before the error propagates can also stop the propagation.

#### Error Propagation

Error propagation in Ciphel can occur in several ways:

1. **Affectation**: Assigning an error to a variable causes that variable to hold the error for its lifetime.
2. **If Statement**: If the condition of an if statement resolves to an error, the chosen branch becomes non-deterministic. Variables modified within that scope inherit the propagated error.
3. **Match Statement**: Similar to if statements, if the expression being matched resolves to an error, the chosen branch becomes non-deterministic, and variables within that scope receive the error.
4. **For Loop**: If an iterator yields an error, the error propagates through the loop's attached scope, affecting the iterator's items.
5. **While Loop**: When the condition resolves to an error, variables created or mutated within the loop's scope can be non-deterministically affected by the error.
6. **Function Calls**: Calling a function with an argument that is an error causes the error to propagate within the function's scope. Variables created or mutated with the error argument will be affected.
7. **Return Statement**: Errors are propagated through the outer scope via return statements.

#### Error in Expressions

- **Binary Operations**: An error combined with any value in a binary operation results in an error.
- **Unary Operations**: Unary operations on an error also result in an error.
- **If Expression**: If the condition is an error, the result of the if expression is an error.
- **Match Expression**: If the value being pattern-matched is an error, the result of the match expression is an error.

#### Channel Propagation

Errors can also be propagated through channels, affecting the communication and synchronization mechanisms within the game.

#### Cell Corruption and Reset

If a cell on the ribbon is corrupted by an error, it gets reset to its default mode, state, and substate.

### Types

The concept of types is central to the definition of the data layout and the semantics of variables. These types are differentiated according to their storage location: the stack and the heap. Each memory type has certain properties that influence gameplay and strategy.

#### Stack Data Types

##### **Primitives, Slices, Units, Tuples, Structs, and Addresses**:

These types are stored directly on the stack. - **Characteristics**: - **Unprotected**: Data stored on the stack is easily accessible to opponents, making it vulnerable to attacks and surveillance. - **Energy Cost**: There is no energy cost associated with storing, reading, or modifying stack data. This makes stack storage efficient but risky due to its lack of protection.

1. **Primitive Types**:
   - `number` (64-bit signed Integer):
     - **Operations**: Supports mathematical (+, -, \*, /, %, <<, >>) and logical (or, and, xor) operations, returning an `number`. Comparison operators (>=, >, <, <=, =\=, !=) yield a `bool`. The negation operator returns true if the uint is strictly greater than 0, otherwise false.
   - `float` (64-bit Floating Point):
     - **Operations**: Mathematical and logical operations return `float`. Comparison operators result in a `bool`. Negation returns `true` if the `float` is non-zero.
   - `char` (1 Byte Unsigned Integer for ASCII Characters):
     - **Operations**: Supports mathematical and logical operations, returning `char`. Comparison operators yield a `bool`. Negation returns `true` if the `char` is non-zero.
   - `bool` (1 Byte Unsigned Integer):
     - **Operations**: Logical operations (or, and, xor) are endomorphic, preserving `bool` type.
2. **Slice**:
   - Fixed-size array of a known and identical type.
     - **Operations**: Indexing (0 to length-1) for element retrieval. Element inclusion results in a `bool`.
   - Static String:
     - ASCII string defined at compile-time.
     - **Operations**: Indexing for character retrieval. Substring inclusion results in a `bool`.
3. **Unit**:
   - Represents a default no-value (1 byte unsigned integer).
     - **Operations**: None.
4. **Tuple**:
   - Fixed-size array of varied types.
     - **Operations**: Indexing (0 to length-1) for element retrieval.
5. **Address**:
   - Represents the address of a value in the heap or stack (1 byte unsigned integer).
     - **Operations**: Mathematical and logical operations result in a potentially invalid address, preserving the type (heap or stack). Comparison operators yield a `bool`.
6. **Struct**:
   - Aligned array of data, accessible by field name.
     - **Operations**: Accessing a field returns the field's offset in the struct, equating to its address in the stack.

#### Heap Data Types

##### **Vectors, Closures, Channels, and Maps**:

These types are exclusively stored on the heap. - **Characteristics**: - **Protected**: Data on the heap is more challenging for opponents to access or corrupt. Players have the option to further protect this data. - **Energy Cost**: Maintaining data on the heap requires energy. Both storing and modifying heap data incur an energy cost. This makes heap storage more secure but at the expense of resource consumption.

1. **Vector**:
   - **Description**: A dynamic array represented by a smart pointer. It contains the address of the first array element in the heap, along with the array's length and capacity.
   - **Operations**:
     - **Length and Capacity**: Retrieve the length and capacity of the vector.
     - **Append**: Add an element at the end of the vector. If the current capacity is insufficient, the vector's capacity is doubled (changing the address of the first element), which incurs an energy penalty.
     - **Remove**: remove an element at the given index
     - **Element Inclusion**: Check if an element is in the vector, returning a `bool`.
2. **Map**:
   - **Description**: A dynamic hashmap, also represented by a smart pointer. It stores key-value pairs, with keys being hashable data of either a primitive or address type. Contains the address of the first hashmap bucket, length, and capacity.
   - **Operations**:
     - **Insert**: Add a value at a specified key. If the capacity is inadequate for a new value, the capacity is doubled (changing the address of the first bucket), leading to an energy penalty.
     - **Contains**: Check if a key exists in the map -> Inclusion overload.
     - **Delete**: Remove a key and its associated value (no action if the key is absent).
     - **Length and Capacity**: Retrieve the length and capacity of the map.
3. **Fn (Closure)**:
   - **Description**: Represents the address of a closure stored on the heap.
     A closure is a combination of a function and an outer scope. This outer scope is distinct in that it is created at the point where the closure is defined.
     - **Outer Scope Creation**: The outer scope of a closure is formed using variables from the scope where the closure was created. These variables are duplicated into the new outer scope.
     - **Variable Access**: Functions within the closure have access to, and can modify, the variables present in the outer scope.
     - **Scope Persistence**: The outer scope of a closure remains active as long as the closure exists. This ensures that each time the function within the closure is called, it operates with the same outer scope context.
   - **Function-Scoped Variables**: Variables used within the function, and defined in the closure's creation scope, are encapsulated within the closure.
   - **Modification and Access**: Function inside a closure can interact with the outer scope's variables, allowing for dynamic data manipulation based on the closure's execution context.
   - **Lifetime Dependency**: The outer scope's lifetime is tied to the closure's existence, ensuring consistency and data integrity every time the function is invoked.
   - **Operations**:
     - **Call**: Execute the closure.
4. **Channel**:
   - **Description**: An address on the heap representing a communication channel.
     A channel is a component for inter-process or event communication. It serves as a two-way communication protocol, facilitating data exchange between different entities within the game.
     - **Two-Way Communication**: Channels enable bidirectional communication, allowing entities to both send and receive data.
     - **Multiple Readers**: A channel can have multiple entities reading from it simultaneously, enabling broad data dissemination and synchronization among different processes or events.
     - **Single Writer**: At any given time, a channel allows only one writer.
   - **Operations**:
     - **Receive**: Read a value from the channel with optional timeout parameters (-1 for infinite wait, 0 for no wait, or a specified number of ticks).
     - **Send**: Write a value to the channel. Returns an error if there is no reader or there is already a writer
