# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

This is a **personal AI agent guidelines repository**. It contains generic programming guidelines that can be referenced by any development project. When working on this repository, you are maintaining documentation and guidelines for AI agents to use in other projects.

## Repository Architecture

### Structure

```
ai-agent-guidelines/
├── rust/
│   ├── guidelines/
│   │   ├── guidelines.md          # Entry point - references language guidelines
│   │   └── language_guidelines.md # Rust coding standards, testing, error handling
│   └── prompt/
│       └── coding_new_feature.md  # AI agent prompt template for new features
├── php/
│   └── guidelines/
│       ├── guidelines.md          # Entry point
│       └── language_guidelines.md # PHP/Symfony coding standards
├── python/
│   └── guidelines/
│       ├── guidelines.md          # Entry point
│       └── language_guidelines.md # Python/FastAPI coding standards
├── typescript/
│   └── guidelines/
│       ├── language_code_style.md          # TypeScript language & code style
│       ├── typescript_code_style.md        # TypeScript patterns (CQRS, Store, etc.)
│       ├── react_guide_line.md             # React component guidelines
│       ├── rtl_testing_guideline.md        # React Testing Library guidelines
│       ├── librairies_guidelines.md        # Library contribution guidelines
│       └── typescript_testing_guideline.md # TypeScript testing guidelines
├── README.md
└── CLAUDE.md
```

### Guidelines Organization

**Entry Point Pattern**: Each language directory contains a `guidelines/guidelines.md` that serves as the main entry point, referencing:
1. Language-specific guidelines (coding standards)
2. Project-specific guidelines (in the consuming project)

## Working on This Repository

### Key Principles

1. **Keep Content Generic**: Guidelines should apply across multiple projects. Remove project-specific references.

2. **Use Placeholders**: Replace specific values with generic placeholders where customization is needed.

3. **Maintain Consistency**: All guidelines use consistent patterns for code examples, command formatting, and structure.

4. **Document for AI Agents**: Write guidelines that AI agents can parse and apply programmatically.

### Editing Guidelines

When updating guideline files:

1. **Language Guidelines** (`language_guidelines.md`):
   - Quick reference commands at the top
   - Pre-commit checklist
   - Code style, anti-patterns, testing patterns
   - Error handling and environment variables

### Common Tasks

**Adding New Language Support**:
1. Create directory: `{language}/`
2. Create structure: `guidelines/`
3. Write `guidelines/guidelines.md` as entry point
4. Create language-specific guidelines
5. Update main README.md

**Updating Guidelines**:
- Ensure all code examples use environment variables (not hardcoded values)
- Include both sync and async test patterns where applicable
- Use appropriate syntax highlighting in code blocks
- Keep examples compilable and realistic

**Testing Changes**:
- Guidelines are documentation only - no builds or tests
- Review for clarity and completeness
- Ensure AI agents can follow instructions unambiguously

## Important Notes

**No Build System**: This repository contains only Markdown documentation - no Cargo.toml, package.json, or other build configuration.
