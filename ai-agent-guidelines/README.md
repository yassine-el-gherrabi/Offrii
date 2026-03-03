# AI Agent Guidelines

Personal collection of programming guidelines for AI agents working on software development projects. Provides standardized coding standards, patterns, and best practices across multiple languages.

## Purpose

This repository centralizes AI agent guidelines to:

- Maintain consistent AI agent behavior across projects
- Share best practices and coding standards
- Reduce duplication of guideline content
- Provide language-specific patterns and conventions

## Repository Structure

```
├── README.md
├── CLAUDE.md
├── rust/
│   ├── guidelines/
│   │   ├── guidelines.md           # Entry point for AI agents
│   │   └── language_guidelines.md  # Rust-specific coding standards
│   └── prompt/
│       └── coding_new_feature.md   # AI agent prompt for new features
├── php/
│   └── guidelines/
│       ├── guidelines.md           # Entry point for AI agents
│       └── language_guidelines.md  # PHP/Symfony coding standards
├── python/
│   └── guidelines/
│       ├── guidelines.md           # Entry point for AI agents
│       └── language_guidelines.md  # Python/FastAPI coding standards
└── typescript/
    └── guidelines/
        ├── language_code_style.md          # TypeScript language & code style
        ├── typescript_code_style.md        # TypeScript patterns (CQRS, Store, etc.)
        ├── react_guide_line.md             # React component guidelines
        ├── rtl_testing_guideline.md        # React Testing Library guidelines
        ├── librairies_guidelines.md        # Library contribution guidelines
        └── typescript_testing_guideline.md # TypeScript testing guidelines
```

## Integration with Projects

To use these guidelines in a project:

1. Copy or reference the relevant language guidelines
2. Create a `project_guidelines.md` in your project root for project-specific patterns
3. Configure your AI agent to read both the generic and project-specific guidelines

### Project Guidelines

Your `project_guidelines.md` should contain:

- Project-specific architecture and structure
- Business logic patterns
- Database schemas and relationships
- API specifications
- Project-specific dependencies and tools

## Guidelines Organization

### Entry Point: `guidelines.md`

Each language folder contains a `guidelines/guidelines.md` file that serves as the entry point for AI agents. This file references:

1. **Language Guidelines** - Programming language-specific standards and best practices
2. **Project Guidelines** - Project-specific information (located in the consuming project)

### Language Guidelines

Contains programming language-specific information:
- Coding standards and conventions
- Testing practices
- Error handling patterns
- Dependency management
- Pre-commit requirements

## Usage for AI Agents

When working on a project that includes these guidelines, AI agents should:

1. **Read the entry point**: Start with `{language}/guidelines/guidelines.md`
2. **Review language guidelines**: Follow links to language-specific guidelines
3. **Check project-specific guidelines**: Read the `project_guidelines.md` in the project root
4. **Apply all guidelines**: Ensure code changes adhere to both generic and project-specific guidelines

## Language Support

Currently supported:
- **Rust** - Cargo workspace projects with tracing, thiserror, and async patterns
- **PHP** - Symfony 7 projects with Doctrine ORM, GrumPHP, and Messenger
- **Python** - FastAPI projects with MongoEngine, dependency-injector, and OpenTelemetry
- **TypeScript** - React projects with CQRS patterns, RTL testing, and strict type safety

## Contributing

When contributing to these guidelines:

1. Keep content generic and applicable across multiple projects
2. Remove any project-specific references
3. Use placeholder patterns where customization is needed
4. Test guidelines with multiple project types before merging
