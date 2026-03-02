# Development Guidelines for Rust Projects

This document serves as a reference for AI agents and developers working on Rust projects. It provides pointers to
the detailed guidelines that should be followed when making changes to the codebase.

## Purpose of This File

This file (guidelines.md) is primarily used by AI agents to complete various tasks in Rust projects. The
content has been organized into separate files to make it easier to find specific guidelines:

1. [Programming Language Guidelines](language_guidelines.md) - Contains Rust-specific coding standards, patterns,
   and best practices

## Guidelines for AI Agents

When working on tasks in Rust projects, AI agents must:

1. Review the Programming Language Guidelines and Project Guidelines (from the consuming project)
2. Follow the Rust programming standards outlined in the language guidelines
3. Respect the project structure and implementation patterns described in the project-specific guidelines
4. Ensure all code changes adhere to all sets of guidelines
5. Run the required pre-commit checks before submitting changes

## Quick Reference

Here are the key files that contain the detailed guidelines:

- [Programming Language Guidelines](language_guidelines.md): Rust coding standards, testing practices, error
  handling, etc.
- [Project Guidelines](../../../project_guidelines.md): Project structure, feature flags, message handling, etc.
  (This file should be located at the root of the consuming project)

For specific implementation details, refer to the appropriate section in these files.
