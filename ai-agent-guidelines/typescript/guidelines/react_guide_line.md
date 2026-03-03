# React Guidelines

## React Import Pattern
Always import React with namespace import and use the `React.` prefix for all hooks.

**Pattern:**
- Import React as: `import * as React from 'react';`
- Use `React.useCallback`, `React.useMemo`, `React.useEffect`, `React.useState`
- Never import individual hooks with destructuring

## Memoization Patterns

- Always use `React.useMemo` for expensive computations.
- Always use `React.useCallback` for stable function references and never pass a non memoized function to child components.

## Context Pattern with Null Support
 - When creating React Context, use `null` as the default value to enable graceful error handling when components are used outside their provider.
 - Create its custom hook that checks for `null` and throws a clear error if the context is accessed without a provider.

## Component Typing Patterns

Use `React.FC` for functional components with explicit prop types when the component has props.
Never use interfaces for component props; prefer `type` aliases instead.

**Empty props pattern:**
- When component has no props, use `type Props = Record<string, never>;`
- Explicitly signals component accepts no props
- More type-safe than empty interface or omitting props entirely

## Translation Pattern

Use `react-intl` with `useIntl` hook for all translations.

**Key characteristics:**
- Never use `defaultMessage` parameter
- Only pass `id` to `intl.formatMessage()`
- All keys must exist in translation files
- Use hierarchical dot notation for keys
- Never use string literals for translations

**Key naming convention:**
- Pattern: `feature.component.element.property`
- Example: `workflows.modal.title`
- Common keys: `common.save`, `common.cancel`, `common.delete`

**When to use:**
- All user-facing text
- Button labels, placeholders, error messages
- Dynamic messages with variables

## Form Submission Pattern

Handle form submission with validation before dispatching actions.

**Key characteristics:**
- Prevent default with `event.preventDefault()`
- Stop propagation with `event.stopPropagation()`
- Check `isFormDataValid` property (not method)
- Call `showErrors()` if validation fails
- Only dispatch action if validation passes

**Pattern:**
- Wrap handler in `React.useCallback`
- Include the formData in dependency array
- Return early if validation fails
- Dispatch command/query after validation

## Component Organization Pattern

Follow Single Responsibility Principle for component structure.

**Key characteristics:**
- Each component handles ONE concern or feature
- Small, focused components are preferred over large multi-purpose ones
- Extract logical sections into separate specialized components
- Name components based on their specific responsibility

**When to use:**
- Complex UI requiring multiple responsibilities
- Reusable UI patterns across different contexts
- Improving testability and maintainability

**Benefits:**
- Easier to understand and maintain
- Better code reuse
- Simpler testing
- Clear component boundaries

## JSX Fragment Pattern

Always use explicit `<React.Fragment>` over shorthand `<>`.

## Conditional Rendering Pattern

Handle component visibility and conditional UI with clear, maintainable patterns.

**Early return pattern:**
- Use early return for component visibility: `if (!isVisible) return null;`
- Alternative: Return loader component for loading states: `if (isLoading) return <Loader />;`

**Ternary vs && operator:**
- Use ternary operator when both branches render something: `condition ? <ComponentA /> : <ComponentB />`
- Use `&&` operator when only truthy branch renders: `condition && <Component />`
- Never use `&&` with numbers or strings that could render unintended values

**Avoid nested ternaries:**
- Nested ternaries are hard to read and maintain
- Extract complex conditions into variables or separate components
- Use early returns or separate render methods for complex logic

## State Update Pattern

Use functional updates when new state depends on previous state.

**Functional update pattern:**
- Always use `setState(prev => ...)` when depending on previous state
- Ensures state updates are based on latest state value
- Prevents race conditions in async updates or multiple rapid updates
