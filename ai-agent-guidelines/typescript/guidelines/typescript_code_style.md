## CQRS Pattern

### Commands (Write Operations)

Commands are plain classes without validation decorators, designed for write operations.

**Key characteristics:**
- Plain classes with NO decorators
- All properties `readonly`
- Use `Object.freeze(this)` in constructor for immutability
- Provide `asJSON` getter for API serialization (converts to snake_case)

**When to use:**
- Write operations in Application layer
- Data transfer objects for API calls
- Immutable command objects for state management

---

### Queries (Read Operations)

Queries are simple data containers with readonly properties.

**Key characteristics:**
- Plain classes with no methods
- Constructor with readonly parameters
- No validation or business logic
- No `Object.freeze` needed (no methods to mutate)

**When to use:**
- Read operations in Application layer
- Fetching data from APIs
- Parameter objects for queries

---

### Actions (State Transitions)

Actions are default export functions returning typed action objects.

**Key characteristics:**
- Export type interface first
- Default export function (not factory function)
- Include `type` constant, `payload`, and context (accountId, botId)
- Transform payload if needed (snake_case → camelCase, date parsing)

**Pattern structure:**
1. Define payload type
2. Define computed payload type (transformed)
3. Define action type interface
4. Export transformation function (if needed)
5. Default export action creator function

**When to use:**
- State transitions in Redux-like stores
- Dispatching changes to application state
- Event-driven architecture

---

### Application Executors

Application executors return async functions accepting `Dispatch` parameter.

**Key characteristics:**
- Import `Dispatch` type from your store manager library
- Return function with signature: `async (dispatch: Dispatch) => void`
- Try-catch block with success and failure action dispatches
- Accept query or command objects as parameters

**Pattern:**
- Create query/command instance
- Pass to executor
- Executor returns async function
- Async function dispatches actions

---

## Store Pattern (SliceReducer)

### Reducer Function Definition

Define reducer functions as separate named functions before passing to `SliceReducer`.

**Key characteristics:**
- Named functions with descriptive names (prefix: `when`)
- Signature: `(state: State, action: Action) => State`
- Return new state object (immutability)
- Define all functions before `SliceReducer` instantiation

**Naming convention:**
- Use `when` prefix: `whenIntentsFetched`, `whenIntentCreated`
- Describe the action being handled
- Use past tense for completed actions

**Why separate functions:**
- Better readability and maintainability
- Reusable across multiple stores
- Easier to test in isolation
- Clear function signatures

---

## View Model Pattern

View models use specific method names for operations, avoiding generic patterns.

**Key characteristics:**
- All properties `readonly`
- `Object.freeze(this)` in constructor
- Specific method names based on operation type
- Methods return new instances (immutability)
- Throw errors for invalid operations

**Method naming conventions:**
- `add*` - Add items to collections (`addStep`, `addIntent`)
- `remove*` - Remove items from collections (`removeStep`, `removeIntent`)
- `update*` - Update existing items (`updateStep`, `updateIntent`)
- `find*` - Search returning nullable (`findStepById`, `findIntentByName`)
- `get*` - Search throwing on not found (`getStepById`, `getIntentByName`)

**When to use:**
- Immutable data structures in UI Store layer
- Domain models with business logic
- Redux-like state management

**Benefits:**
- Clear method intent vs generic `withX` pattern
- Better IDE autocomplete and discoverability
- Easier to understand domain operations
- Type-safe immutability

---

## Form Validation Pattern

### Decorator Usage

Use `@validatedClass()` decorator (lowercase with parentheses) for form data classes.

**Key characteristics:**
- Import `validatedClass` (lowercase), not `ValidatedClass`
- Decorate class with `@validatedClass()` including parentheses
- Use `@constraint` decorators with Joi schemas and translation keys
- Extend `AbstractValidatedClass`
- Call `super(data)` first in constructor
- Use `Object.freeze(this)` for immutability

**Validation decorator pattern:**
- Multiple `@constraint` decorators per property
- First parameter: Joi schema
- Second parameter: Translation key for error message
- Order matters: applied from bottom to top

---

### Immutable Setter Methods

Form data classes provide immutable setter methods returning new instances.

**Naming pattern:**
- Prefix with `set`: `setLabel`, `setType`, `setValue`
- Take single parameter of property type
- Return new instance of same class
- Spread existing instance data with new value

**Why immutable setters:**
- Maintains immutability contract
- Works with React state management
- Enables time-travel debugging
- Prevents accidental mutations

---

## Import Patterns

### Import Order

Follow strict import order enforced by ESLint.

**Order:**
1. React imports (`import * as React from 'react'`)
2. External library imports (react-router-dom, third-party UI libraries)
3. Internal/shared library imports (organization-scoped packages)
4. Alias imports (`@Application/*`, `@Domain/*`, `@Infra/*`, `@UI/*`)
5. Relative imports (`./components`, `../utils`)

**Why this order:**
- Clear separation of concerns
- Easy to identify dependencies
- Consistent across codebase
- Enforced by tooling

---

### Path Aliases

Use configured path aliases for absolute imports within project layers.

**Available aliases:**
- `@Application/*` - Application layer (Commands, Queries, Actions, executors)
- `@Domain/*` - Domain layer (Pure functions, utilities, formatters)
- `@Infra/*` - Infrastructure layer (HTTP clients, storage adapters)
- `@UI/*` - UI layer (Components, Store, Router, visualization)
- `@Guidelines/*` - Shared components library
- `@Testing/*` - Test utilities and helpers
- `@Mocks/*` - API mocks for testing

**Benefits:**
- No relative path traversal (`../../../`)
- Refactoring-friendly
- Clear layer boundaries
- IDE autocomplete support

---

## Type Conventions

### Naming Conventions

Follow TypeScript naming conventions for consistency.

**Rules:**
- **Interfaces**: PascalCase, no `I` prefix (e.g., `BotVariable`, not `IBotVariable`)
- **Types**: PascalCase (e.g., `Action`, `State`, `WorkflowType`)
- **Enums**: PascalCase (e.g., `StepType`, `WorkflowUsage`)
- **Constants**: UPPER_SNAKE_CASE for true constants (e.g., `MAX_RETRIES`)
- **Variables/Functions**: camelCase (e.g., `fetchBotDetails`, `isValid`)
- **Classes**: PascalCase (e.g., `CreateBotVariable`, `WorkflowDetailsView`)
- **Private fields**: Prefix with `#` or use `private` keyword

---

### Type vs Interface

Prefer `type` aliases for component props and union types.

**Use `type` for:**
- Component props
- Union types (`type Status = 'pending' | 'active'`)
- Intersection types
- Function signatures
- Mapped types
- API contracts

**Use `interface` for:**
- Object shapes that may be extended
- Class contracts

**React component props:**
- Always use `type`, never `interface`
- Enables better composition with union and intersection types

---

## Immutability Patterns

### Object.freeze for Immutability

Use `Object.freeze(this)` in constructor to enforce immutability at runtime.
And prefer passing a single object parameter to constructors when the class has many properties.

**Pattern:**
1. Assign all properties in constructor
2. Call `Object.freeze(this)` as last statement
3. Prevents any property mutations
4. Works with `readonly` keyword for compile-time safety

**When to use:**
- View models in UI Store
- Commands and Queries in Application layer
- Form data classes
- Any immutable domain object

---

### Readonly Properties

Use `readonly` keyword for all properties in immutable classes.

**Pattern:**
- Declare all properties as `public readonly`
- Assign once in constructor
- TypeScript prevents reassignment at compile time
- Combine with `Object.freeze` for runtime enforcement

---

## Error Handling

### Explicit Error Throwing

Throw errors explicitly for invalid operations with descriptive messages.
Use the throw only when it is necessary as we are in Frontend code

**Pattern:**
- Check preconditions before operations
- Throw `Error` with descriptive message
- Include relevant context in error message

**When to throw:**
- Item not found in collection (e.g., `getStepById`)
- Precondition violations
- Invalid arguments

---

## Before Submitting or when finishing a task

**Run these commands:**

```bash
yarn prettier-fix     # Auto-format code
yarn lint-fix         # Auto-fix linting issues
yarn ts-check         # TypeScript type checking
yarn test-dev         # Run all tests
```

**TypeScript checklist:**

- [ ] All properties `readonly` in immutable classes
- [ ] `Object.freeze(this)` in all immutable constructors
- [ ] Commands are plain classes (no `@validatedClass`)
- [ ] Form data uses `@validatedClass()` (lowercase)
- [ ] Actions are default export functions
- [ ] Reducer functions defined separately
- [ ] View models use specific method names (not `withX`)
- [ ] Import order follows ESLint rules
- [ ] Type vs interface used appropriately
- [ ] Error messages are descriptive
- [ ] No `any` types (use `unknown` instead)
- [ ] Type checking passes