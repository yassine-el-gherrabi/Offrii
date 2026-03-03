# TypeScript Testing Guidelines

## Test File Organization

Structure test files alongside source code for discoverability and maintenance.

**File placement:**
- Place test files next to the source file being tested
- Use `.spec.ts` or `.spec.tsx` extension for test files
- Match the source file name exactly (e.g., `UserService.ts` → `UserService.spec.ts`)

**Directory structure:**
- Keep tests in same directory as source code
- Avoid separate `__tests__` directories unless project convention requires it
- Use path aliases consistently in test imports

---

## Test Structure Pattern (AAA)

Follow Arrange-Act-Assert pattern for clear, maintainable tests.

**Three-phase structure:**
1. **Arrange**: Set up test data, mocks, and dependencies
2. **Act**: Execute the behavior under test
3. **Assert**: Verify expected outcomes

**When to use:**
- All unit and integration tests
- Component tests with user interactions
- API and service layer tests

---

## Describe Block Organization

Organize tests hierarchically using nested `describe` blocks.

**Organization pattern:**
- Top-level `describe`: Feature or class name
- Nested `describe`: Method name or scenario
- Further nesting: Specific conditions or states
- `it` statements: Individual test cases

**Naming conventions:**
- `describe('FeatureName', ...)` for top level
- `describe('methodName', ...)` for methods
- `describe('when condition', ...)` for scenarios
- `it('should behavior', ...)` for test cases

**When to use nested describes:**
- Methods with multiple parameters or conditions
- Complex state combinations
- Testing behavioral boundaries
- Multiple success and failure paths

---

## Test Setup and Teardown

Use Jest lifecycle hooks for consistent test environment.

**beforeEach pattern:**
- Reset all mocks before each test
- Initialize common test data
- Set up fresh instances of objects under test
- Configure default mock return values

**afterEach pattern:**
- Restore mocked functions
- Clean up resources
- Reset global state
- Clear timers and intervals

**When to use:**
- Shared setup across multiple tests
- Cleaning up side effects
- Ensuring test isolation
- Preventing test interdependence

---

## Mocking Strategies

Mock external dependencies appropriately based on test scope.

**Module mocking:**
- Use `jest.mock()` for external modules
- Mock at module level, not within tests
- Provide type-safe mock implementations
- Mock only what's necessary for the test

**Function spying:**
- Use `jest.spyOn()` for method interception
- Verify function calls and arguments
- Restore spies in `afterEach` or after test
- Use spies to track side effects

**Manual mocking:**
- Create mock factories for complex objects
- Use test utilities for consistent mocking
- Provide minimal mock implementations
- Type mocks properly with TypeScript

**When to mock:**
- External APIs and HTTP clients
- Database connections
- File system operations
- Time-dependent functions
- Third-party libraries

---

## Testing Immutable Patterns

Verify immutability through instance identity and state assertions.

**Key assertions:**
- New instance created (different reference)
- Correct type preserved
- New state applied correctly
- Original instance unchanged

**When to test:**
- Immutable class methods
- Redux reducer-like patterns
- Functional state updates
- Builder pattern implementations

---

## Testing Edge Cases

Cover boundary conditions and error scenarios.

**Common edge cases:**
- Null and undefined values
- Empty collections (arrays, objects, strings)
- Boundary values (0, -1, MAX_INT)
- Invalid input types
- Missing required parameters

**Error scenarios:**
- Invalid state transitions
- Precondition violations
- Network failures
- Timeout conditions
- Permission denials

**When to test:**
- Public API methods
- User input validation
- Data transformation functions
- State management logic

---

## Type Safety in Tests

Leverage TypeScript for type-safe tests.

**Type usage:**
- Type test data properly
- Use `as` sparingly, prefer type guards
- Type mock return values
- Avoid `any` in test code

**Type inference:**
- Let TypeScript infer when obvious
- Explicit types for complex test data
- Type test helpers and utilities
- Generic test factories when appropriate

---

## Test Isolation

Ensure tests are independent and repeatable.

**Isolation principles:**
- Each test runs independently
- No shared mutable state between tests
- Tests can run in any order
- No dependencies between test files

**Common isolation issues:**
- Global state mutations
- Singleton pattern usage
- Module-level caching
- Shared mock state

**Solutions:**
- Reset mocks in `beforeEach`
- Create new instances per test
- Use factories for test data
- Clear caches between tests

---

## Test Naming Conventions

Write descriptive, consistent test names.

**Test name pattern:**
- Start with "should"
- Describe expected behavior
- Include relevant context
- Be specific, not generic

**Good examples:**
- `should return null when user not found`
- `should throw error for invalid email format`
- `should update state after successful API call`

**Bad examples:**
- `test 1`
- `works correctly`
- `handles error`

---

## Async Testing

Handle asynchronous operations properly in tests.

**Async patterns:**
- Use `async/await` for promise-based code
- Return promises from test functions
- Use `done` callback for callback-based code
- Handle rejections explicitly

**Timeout handling:**
- Set appropriate timeouts for slow operations
- Use `jest.setTimeout()` for specific tests
- Avoid arbitrary waits
- Mock time-dependent operations

**Common pitfalls:**
- Forgetting to await promises
- Not handling promise rejections
- Race conditions in assertions
- Infinite promise chains

---