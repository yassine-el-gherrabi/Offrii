# TypeScript Language & Code Style Guidelines

## Naming Conventions

Follow TypeScript naming conventions for consistency across the codebase.

**Rules:**
- **Interfaces**: PascalCase, no `I` prefix (e.g., `BotVariable`, not `IBotVariable`)
- **Types**: PascalCase (e.g., `Action`, `State`, `WorkflowType`)
- **Enums**: PascalCase (e.g., `StepType`, `WorkflowUsage`)
- **Constants**: UPPER_SNAKE_CASE for true constants (e.g., `MAX_RETRIES`)
- **Variables/Functions**: camelCase (e.g., `fetchBotDetails`, `isValid`)
- **Classes**: PascalCase (e.g., `CreateBotVariable`, `WorkflowDetailsView`)
- **Private fields**: Prefix with `#` or use `private` keyword

---

## Import Organization

Organize imports in strict order for consistency and maintainability.

**Import order:**
1. React imports (`import * as React from 'react'`)
2. External library imports (react-router-dom, third-party UI libraries)
3. Internal/shared library imports (organization-scoped packages)
4. Alias imports (`@Application/*`, `@Domain/*`, `@Infra/*`, `@UI/*`)
5. Relative imports (`./components`, `../utils`)

**Import grouping:**
- Blank line between each group
- Alphabetical sorting within each group
- Type-only imports can be grouped together or with their source

---

## Type Safety Principles

Maintain strong type safety throughout the codebase.

**Return type annotations:**
- Always explicit for public methods and exported functions
- Explicit for complex functions with non-obvious returns
- Can be omitted for simple, obvious functions where inference is clear
- Never use `any`.

**Type narrowing:**
- Use type guards for runtime validation
- Prefer early returns for null/undefined checks
- Validate inputs at boundaries (API responses, user input)

---

## Code Reusability

Follow DRY (Don't Repeat Yourself) principle through extraction and abstraction.

**When to extract:**
- Logic repeated more than twice
- Complex validation or transformation patterns
- Shared business rules across components

**Extraction patterns:**
- Pure functions for reusable logic
- Custom hooks for React-specific logic with state
- Utility modules for domain-agnostic operations
- Higher-order functions for function composition

---

## Performance Considerations

Write performant code through conscious algorithm and pattern choices.

**Short-circuit evaluation:**
- Use `Array.every()` when checking all items must satisfy condition (stops at first false)
- Use `Array.some()` when checking if any item satisfies condition (stops at first true)
- Avoid `Array.map()` then `Array.every()` pattern (double iteration)

**Early returns:**
- Return early from functions when conditions aren't met
- Avoid computing values that won't be used
- Place expensive computations after cheap validations

**Memoization awareness:**
- Use `React.useMemo` for expensive computations
- Use `React.useCallback` for stable function references
- Understand when re-renders trigger recalculation

---

## Code Documentation

Document only when necessary, prefer self-explanatory code.

**When to add JSDoc comments:**
- Public API functions and exported utilities
- Complex algorithms with non-obvious logic
- Functions with nuanced behavior or edge cases
- Business rules requiring context

**When NOT to add comments:**
- Self-explanatory code with clear naming
- Private implementation details
- Simple getters/setters
- Code that merely restates what the code does

**JSDoc structure:**
- Brief description of purpose
- `@param` tags for parameters
- `@returns` tag for return value
- `@example` for usage patterns when helpful

**Benefits:**
- IDE intelligence and autocomplete
- Generated documentation
- Onboarding efficiency
- API understanding

---

## Code Organization

Structure code logically for maintainability and discoverability.

**File organization:**
- One primary export per file
- Group related utilities in modules
- Keep files focused and cohesive
- Use index files for clean re-exports

**Function organization:**
- Pure functions before impure functions
- Public API before private helpers
- Simple functions before complex functions
- Related functions grouped together

**Class organization:**
- Public properties before private
- Constructor first
- Public methods before private methods
- Group related methods together

**Benefits:**
- Predictable code location
- Easier navigation
- Better code review experience
- Consistent structure across codebase

---

## Type vs Interface

Prefer `type` aliases for most definitions, use `interface` sparingly.

**Use `type` for:**
- Component props (always)
- Union types
- Intersection types
- Function signatures
- Mapped types
- API response contracts

**Use `interface` for:**
- Object shapes that may be extended by consumers
- Class contracts that will be implemented

---

## Constants and Configuration

Define constants for magic values and configuration.

**Constant usage:**
- Extract magic numbers and strings to named constants
- Use UPPER_SNAKE_CASE for true constants
- Group related constants in objects or enums
- Place constants near their usage or in dedicated config files

**Configuration organization:**
- Environment-specific config in separate files
- Type-safe configuration objects
- Validation for required configuration
- Default values for optional configuration

---

## Avoid Common Anti-Patterns

Recognize and avoid TypeScript anti-patterns.

**Type assertions:**
- Avoid `as` type assertions when possible
- Use type guards instead of assertions
- Only assert when you have more information than TypeScript

**Any type:**
- Never use `any` as default
- Use `unknown` when type is truly unknown
- Use proper typing even if verbose
- Document why `any` is necessary if unavoidable

**Type duplication:**
- Extract common types to shared modules
- Use utility types (`Pick`, `Omit`, `Partial`) to derive types
- Reference source of truth types

**Optional chaining overuse:**
- Don't use `?.` everywhere to silence errors
- Fix the root cause of potential undefined
- Use proper type narrowing instead