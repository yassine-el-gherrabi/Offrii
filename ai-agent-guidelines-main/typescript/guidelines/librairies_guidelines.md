## Library Contribution Guidelines

### README Examples

**Use real production patterns:**

```typescript
// ✅ GOOD: Based on actual consumer code
@validatedClass()
class CreateBotVersionFormData extends AbstractValidatedClass {
  @constraint(Joi.string().required(), 'header.createBotVersionModal.title.validation.isRequired')
  @constraint(Joi.string().max(25).allow(null, ''), 'header.createBotVersionModal.title.validation.maxLengthError')
  public readonly title: string;
  // ... rest from real code
}

// ❌ BAD: Synthetic example
@validatedClass()
class ExampleForm extends AbstractValidatedClass {
  @constraint(Joi.string(), 'error')
  public field: string;
}
```

## Versioning

**Always increment the version in `package.json`** when making changes.

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR (x.0.0)**: Breaking changes to public API
    - Example: Removing methods, changing method signatures, removing exports
    - Requires: Migration guide in CHANGELOG

- **MINOR (0.x.0)**: New features, backward compatible
    - Example: Adding new validators, adding optional parameters, new exports
    - Safe to upgrade without code changes

- **PATCH (0.0.x)**: Bug fixes, backward compatible
    - Example: Fixing edge cases, correcting validation logic
    - Safe to upgrade without code changes

### Version Bump Examples

```json
// Adding a new validator (MINOR)
"version": "0.3.2" → "version": "0.4.0"

// Fixing a bug in validateJson (PATCH)
"version": "0.3.2" → "version": "0.3.3"

// Removing deprecated API (MAJOR)
"version": "0.3.2" → "version": "1.0.0"
```

**Checklist:**

- [ ] Version incremented in `package.json` (following semver)
- [ ] Documentation updated (README.md)
- [ ] Examples added to README if new public API