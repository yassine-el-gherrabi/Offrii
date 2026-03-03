# React Testing Library (RTL) Guidelines

## Project Testing Setup

Understand the testing infrastructure before writing tests.

**Pre-configured mocks:**
- `react-intl` is automatically mocked via `mockModules.ts`
- `useIntl` returns `mockIntl()` with formatted translation keys: `[[translation.key]]`
- No need to manually mock `useIntl` in individual test files
- `IntlProvider` automatically passes children through without wrapping

---

## Test Wrapper Pattern

Use the centralized Wrapper component for consistent provider setup.

**Wrapper component:**
- Located at `@Testing/Functional/Wrapper`
- Provides all necessary providers for the project components
- Includes: StateManager, Authorization, MessageBroker, Theme, Router
- Authorization automatically includes all permissions for testing

**When to use:**
- Pass `{ wrapper: Wrapper }` to `render()` options
- Required for components using any project-specific context
- Ensures all providers are configured consistently
- Use for all component tests unless testing provider isolation

**When NOT to use:**
- When testing components that do not use any context
- Testing provider components themselves
- Testing custom provider behavior
- Need specific provider configuration per test

---

## mockComponent Utility

Use `mockComponent` from `@Testing/mockComponent` for child component mocking.

**Basic usage:**
- Import: `import mockComponent from '@Testing/mockComponent'`
- Mock child components to isolate component under test
- Mock at module level with `jest.mock()`
- Renders component as div with `data-mocked`, `data-component`, `data-props` attributes

**Mock definition:**
```typescript
jest.mock('@UI/RestrictedArea/Header/BackToHost', () => mockComponent('BackToHost'));
jest.mock('your-ui-kit', () => ({
    ...jest.requireActual('your-ui-kit'),
    FormInput: mockComponent('FormInput'),
}));
```

**Accessing mocked components:**
- Import `mockedComponents` from `@Testing/mockComponent`
- Use `mockedComponents.retrieve('ComponentName')` for single instance but this will throw if not found
- Use `mockedComponents.retrieveAll('ComponentName')` for multiple instances
- Access props via `.props` property
- Simulate events via `.simulate('eventName', ...args)`

**Benefits:**
- Fast, shallow component tests
- Easy prop verification
- Event simulation without full rendering
- Reduced test complexity

---

## Translation Testing Pattern

Test components with translated text using mocked translation keys.

**Translation key format:**
- Mocked `useIntl().formatMessage()` returns: `[[translation.key]]`
- Use exact key format in assertions
- Example: `getByText('[[header.buttons.create]]')`

**When to use:**
- Finding buttons/text by translation key
- Testing conditional translations
- Dynamic message formatting

---

## Store Hook Mocking

Mock custom store hooks for isolated component testing.

**Store hook mocking pattern:**
- Mock store hooks at module level
- Return mock state objects in `beforeEach`
- Use actual View/Model classes for realistic data
- Mock `useDispatch` separately when testing actions

**Example patterns:**
```typescript
jest.mock('@UI/Store/EntityDetails');
jest.mock('@UI/Store/WorkflowDetails');
jest.mock('your-store-manager', () => ({
    ...jest.requireActual('your-store-manager'),
    useDispatch: jest.fn(),
}));

beforeEach(() => {
    (useEntityDetails as jest.Mock).mockReturnValue({
        entity: new EntityView(/* ... */),
    });
    (useDispatch as jest.Mock).mockReturnValue(jest.fn());
});
```

**When to use:**
- Testing components consuming store state
- Isolating component from store implementation
- Testing different state scenarios
- Verifying dispatch calls

---

## Router Hook Mocking

Mock router hooks for components with navigation.

**Router hook mocking:**
- Mock `react-router-dom` at module level
- Spread actual implementation: `...jest.requireActual('react-router-dom')`
- Mock specific hooks: `useMatch`, `useNavigate`, `useParams`
- Configure return values in `beforeEach` per test scenario

**Example pattern:**
```typescript
jest.mock('react-router-dom', () => ({
    ...jest.requireActual('react-router-dom'),
    useMatch: jest.fn(),
}));

beforeEach(() => {
    (useMatch as jest.Mock).mockReturnValue({
        params: { workflowId: '1', botVersionId: 'version-1' },
    });
});
```

**When to use:**
- Components using route parameters
- Testing navigation behavior
- Conditional rendering based on route
- Testing route matching logic

---

## Custom Hook Mocking

Mock project-specific custom hooks appropriately.

**Custom hook patterns:**
- Mock at module level with full path
- Return realistic data structures
- Mock in `beforeEach` for test-specific values
- Use actual classes for type safety

**Common hooks to mock:**
- `useSettings` - Application configuration
- `useEnvironmentContext` - Environment data
- Custom store selectors
- Feature-specific hooks

**Example:**
```typescript
jest.mock('@UI/SettingsManagement');
jest.mock('@UI/Router/Hooks/useEnvironmentContext');

beforeEach(() => {
    (useSettings as jest.Mock).mockReturnValue({
        locale: 'en',
        entityId: 'entityId',
        accountId: 1,
    });
});
```

**Benefits:**
- Isolated hook testing
- Controlled hook return values
- Test-specific configurations
- Reduced dependencies

---

## Snapshot Testing Strategy

Use snapshots for stable UI structure verification.

**Snapshot usage:**
- Use `asFragment().toMatchSnapshot()` for full component structure
- Use `document.body.toMatchSnapshot()` for modal/portal content
- Create snapshots for different states/props
- Review snapshots carefully during PR reviews

**When to snapshot:**
- Initial component rendering
- Different prop combinations
- Conditional rendering branches
- Complex layout structures

---

## Testing User Interactions

Test user interactions with appropriate RTL patterns.

**Click interactions:**
- Use `getByText()` to find interactive elements
- Use `fireEvent.click()` for click simulation
- Wrap in `act()` for state updates
- Use `await act(async () => { ... })` for async updates

**Form interactions:**
- Access mocked inputs via `mockedComponents.retrieve('FormInput')`
- Call `props.onChange()` directly on mocked components
- Use `act()` when triggering state changes
- Verify prop updates after changes

**When to use which:**
- Real elements: `fireEvent` or `userEvent`
- Mocked components: Call `props.onChange/onClick` directly
- Async operations: Wrap in `act()`
- State updates: Always use `act()`

---

## Testing Async Operations

Handle asynchronous operations correctly in tests.

**Async patterns:**
- Wrap async operations in `act(async () => { ... })`
- Use `await` for promises
- Use `waitFor` from RTL for conditional assertions
- Handle modal/dialog opening with async timing

**Common scenarios:**
- Button clicks opening modals
- Form submissions
- Data fetching (mocked)
- State updates after events

**Example:**
```typescript
await act(async () => {
    getByText('[[button.label]]').click();
});

expect(getByText('[[modal.title]]')).toBeInTheDocument();
```

---

## Testing Modal Components

Test modals with proper visibility and interaction patterns.

**Modal testing:**
- Test both open and closed states
- Use `document.body` snapshot for portal content
- Test opening trigger and closing behavior
- Verify modal content when open

**Visibility testing:**
- Pass `open={false}` prop for closed state
- Pass `open={true}` prop for open state
- Test conditional rendering based on `open` prop
- Verify modal not in document when closed

---

## Accessing Mocked Component Props

Verify props passed to mocked child components.

**Property access:**
- Use `mockedComponents.retrieve('ComponentName')` for single component
- Use `mockedComponents.retrieveAll('ComponentName')[index]` for multiple
- Access props via `.props` property
- Access specific prop via `.prop('propName')`

**Example:**
```typescript
const titleInput = mockedComponents.retrieveAll('FormInput')[0];
expect(titleInput.props.disabled).toBe(true);
expect(titleInput.props.value).toBe('intentName');
```

**When to use:**
- Verifying prop values
- Testing conditional props
- Testing prop changes after interactions
- Accessing nested component props

---

## Testing Form Submissions

Test form submission behavior and command dispatching.

**Submission pattern:**
- Mock `useDispatch` from store manager
- Create expected command object
- Trigger submit button click
- Verify dispatch called with correct command

**Example:**
```typescript
const dispatch = useDispatch();
const { getByText } = render(<Form />, { wrapper: Wrapper });

fireEvent.click(getByText('[[form.submit]]'));

expect(dispatch).toHaveBeenCalled();
expect(updateIntent).toHaveBeenCalledWith(
    expect.objectContaining({
        intentId: 'intentId',
        botId: 'botId',
    })
);
```

---

## Test Organization

Structure tests following consistent patterns.

**Test structure:**
- Top-level `describe`: Component name
- Nested `describe`: Feature area (render, interactions, submit)
- Use `beforeEach` for common setup
- Use `beforeAll` for one-time setup
- Reset mocks in `beforeAll` or `beforeEach`

**Describe grouping:**
- `describe('render', ...)` - Rendering tests
- `describe('change formData', ...)` - Form interaction tests
- `describe('submit form', ...)` - Submission tests
- `describe('when condition', ...)` - Conditional behavior

**Benefits:**
- Clear test organization
- Easy navigation
- Logical grouping
- Consistent patterns

---

## Common Test Patterns

Follow established testing patterns for consistency.

**Setup pattern:**
```typescript
beforeEach(() => {
    jest.resetAllMocks();
    // Configure mocks
});
```

**Rendering pattern:**
```typescript
const { getByText, asFragment } = render(
    <Component />,
    { wrapper: Wrapper }
);
```

**Assertion pattern:**
```typescript
expect(element).toBeInTheDocument();
expect(element).toMatchSnapshot();
expect(mockFn).toHaveBeenCalledWith(expectedArgs);
```
