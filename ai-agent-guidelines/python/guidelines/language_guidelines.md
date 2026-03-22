# Python Programming Guidelines

This document provides guidelines for writing Python code in this project.

## Quick Reference

### Pre-commit Checklist
```bash
# Using Task (recommended)
task lint              # Run ruff linter
task format            # Run ruff formatter
task unused            # Find unused code with vulture

# Or directly
ruff check src/        # Lint code
ruff format src/       # Format code
pytest                 # Run tests
```

### Running Tests
```bash
# All tests
pytest

# Unit tests only
pytest tests/unit

# Functional tests
pytest tests/functional

# With verbose output
pytest -v

# Specific test
pytest tests/unit/entity/steps/test_obfuscation.py::TestExclusionRule::test_default_values
```

### Common Environment Variables
- `APP_ENV` - Application environment (dev, staging, prod)
- `PYTHONPATH` - Python path (set to `/code` in Docker)
- `MONGODB_HOST` - MongoDB connection string
- `OTEL_SERVICE_NAME` - OpenTelemetry service name
- `OTEL_PYTHON_EXCLUDED_URLS` - URLs excluded from tracing (e.g., healthcheck)

## Guidelines for Agentic Contributors

When working as an AI agent on this project, you may need to examine source code from external dependencies to understand their implementation or debug issues.

### External Library Source Code Locations

- **Virtual environment**: `.venv/lib/python3.11/site-packages/`
- **Docker container**: `/usr/local/lib/python3.11/site-packages/`
- **Git dependencies**: Installed from Git repositories

## Target Project Structure

These guidelines apply to Python FastAPI projects with AI/LLM functionality. The project structure follows this organization:

- **src/api/** - FastAPI application and endpoints
- **src/entity/** - Domain entities (MongoEngine documents)
  - **src/entity/steps/** - Step entities for AI processing pipeline
- **src/services/** - Business logic services
- **src/dependency_injection/** - Dependency injection containers
- **src/payloads/** - Request/response models (Pydantic)
- **src/models/** - Data models
- **src/utils/** - Utility functions
- **src/scripts/** - Utility scripts
- **tests/unit/** - Unit tests (mirror src/ structure)
- **tests/functional/** - Functional/integration tests
  - **tests/functional/api/** - API endpoint tests

**Note**: When working on a Python project that includes these guidelines, you should apply the conventions described here.

## Code Style

The project uses Python 3.11 with FastAPI framework and follows PEP 8 standards.

### Python Configuration

```txt
# requirements.txt
fastapi==0.109.2
pydantic==2.10.5
uvicorn==0.32.0
gunicorn==23.0.0
mongoengine==0.29.1
pymongo[srv]==4.10.1
dependency-injector==4.48.2
langchain==0.3.7
langchain-openai==0.2.5
opentelemetry-api==1.29.0
opentelemetry-sdk==1.29.0
opentelemetry-instrumentation-fastapi==0.50b0
pytest==8.4.2
```

### Required Development Tools

The project uses these tools (configured via Taskfile):

- **pytest**: Testing framework
- **ruff**: Fast Python linter and formatter (replaces black, isort, flake8)
- **vulture**: Dead code finder
- **docker-compose**: Local development environment

## Identify and Avoid Common Anti-Patterns

Before implementing your plan, verify you don't introduce these anti-patterns:

* **Using mutable default arguments** — Leads to shared state bugs. Use `None` and initialize in function body.
* **Bare except clauses** — Catches all exceptions including system exits. Always specify exception types.
* **Not using context managers** — Leads to resource leaks. Always use `with` for files, connections, locks.
* **Not using type hints** — Reduces code clarity. Use type hints for function signatures.
* **Using `import *`** — Creates namespace pollution. Always import explicitly.
* **Not using Pydantic for validation** — Leads to manual validation code. Use Pydantic models.
* **Mixing sync and async incorrectly** — Causes blocking. FastAPI endpoints can be sync or async.
* **Not using pathlib** — String manipulation for paths is error-prone. Use `Path` from `pathlib`.
* **Global mutable state** — Breaks testability. Use dependency injection.
* **Not using MongoEngine validators** — Skips field validation. Use MongoEngine field types and validators.

## Pre-commit Requirements

The project uses **Ruff** for linting and formatting, managed via Taskfile.

### 1. Linting with Ruff

Run linter to check code quality:

```bash
task lint
# Or: ruff check src/
```

Fix issues automatically:

```bash
task lint:fix
# Or: ruff check --fix src/
```

Ruff replaces multiple tools:
- **Pyflakes**: Error detection
- **pycodestyle**: PEP 8 compliance
- **isort**: Import sorting
- **flake8-bugbear**: Bug detection
- **pyupgrade**: Modern Python syntax

### 2. Formatting with Ruff

Format code automatically:

```bash
task format
# Or: ruff format src/
```

Ruff formatter is compatible with Black but faster.

### 3. Dead Code Detection

Find unused code:

```bash
task unused
# Or: vulture src/ --min-confidence 80
```

### 4. Run Tests

Always run tests before committing:

```bash
pytest
```

## MongoEngine Models

The project uses MongoEngine as MongoDB ODM. Follow these patterns:

### Defining MongoEngine Documents

```python
from mongoengine import (
    Document,
    EmbeddedDocument,
    StringField,
    IntField,
    BooleanField,
    ListField,
    ReferenceField,
    EmbeddedDocumentField,
)

class BotConfiguration(EmbeddedDocument):
    context_prompt = StringField(
        required=False,
        description="Prompt injected at the beginning for global context"
    )
    brand_name = StringField(
        required=False,
        description="The name of the brand"
    )
    language_detection = BooleanField(
        required=False,
        default=False,
        description="Enable or disable language detection"
    )

class Sequence(Document):
    """Represents an AI processing sequence."""

    name = StringField(required=True, unique=True)
    account_id = IntField()
    bot_configuration = EmbeddedDocumentField(BotConfiguration)
    steps = ListField(ReferenceField('Step'))

    meta = {
        'collection': 'sequences',
        'indexes': [
            'name',
            'account_id',
        ]
    }
```

### MongoEngine Field Validation

```python
from mongoengine import Document, StringField, IntField, ValidationError

class ExclusionRule(EmbeddedDocument):
    entity_group_pattern = StringField()
    value_pattern = StringField()
    match_mode = StringField(default="STRICT", choices=["STRICT", "REGEX"])
    normalize_unicode = BooleanField(default=True)

    @staticmethod
    def validate_tolerance(value):
        """Custom validator for tolerance field."""
        if value is None or isinstance(value, bool):
            return value
        if isinstance(value, Tolerance):
            return value
        raise ValidationError("Invalid tolerance value")

    def clean(self):
        """Called before save() to validate document."""
        if self.value_pattern and not self.entity_group_pattern:
            raise ValidationError(
                "entity_group_pattern required when value_pattern is set"
            )
```

### Best Practices for MongoEngine

- Use `required=True` for mandatory fields
- Provide `description` for documentation
- Use `default` for optional fields with defaults
- Define `meta` for collection name and indexes
- Use `EmbeddedDocument` for nested structures
- Use `ReferenceField` for relationships
- Implement `clean()` for cross-field validation
- Use `@staticmethod` validators for field-level validation

## FastAPI Application Structure

### Application Initialization

```python
from dependency_injector import containers
from fastapi import FastAPI
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

from src.api import endpoints
from src.dependency_injection import build_container

def create_app() -> FastAPI:
    """Create and configure FastAPI application."""
    rest_container = containers.DynamicContainer()
    rest_app = FastAPI()
    rest_app.container = build_container(rest_container)
    rest_app.include_router(endpoints.router)

    # Instrument with OpenTelemetry
    FastAPIInstrumentor.instrument_app(rest_app)

    return rest_app

rest_api = create_app()
```

### API Endpoints

```python
from fastapi import APIRouter, Depends, HTTPException
from dependency_injector.wiring import inject, Provide

from src.payloads.sequence import SequenceRequest, SequenceResponse
from src.services.sequence_service import SequenceService

router = APIRouter(prefix="/api/v1", tags=["sequences"])

@router.post("/sequences", response_model=SequenceResponse)
@inject
async def create_sequence(
    request: SequenceRequest,
    service: SequenceService = Depends(Provide["services_container.sequence_service"])
) -> SequenceResponse:
    """Create a new sequence."""
    try:
        sequence = service.create(request)
        return SequenceResponse.from_entity(sequence)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/healthcheck")
async def healthcheck():
    """Health check endpoint."""
    return {"status": "healthy"}
```

### Pydantic Models for Payloads

```python
from pydantic import BaseModel, Field, validator
from typing import Optional, List

class SequenceRequest(BaseModel):
    """Request payload for creating a sequence."""

    name: str = Field(..., min_length=1, max_length=100)
    account_id: int = Field(..., gt=0)
    context_prompt: Optional[str] = None
    language_detection: bool = Field(default=False)

    @validator('name')
    def validate_name(cls, v: str) -> str:
        """Validate sequence name."""
        if not v.strip():
            raise ValueError("Name cannot be empty")
        return v.strip()

    class Config:
        json_schema_extra = {
            "example": {
                "name": "customer-support",
                "account_id": 123,
                "language_detection": True
            }
        }

class SequenceResponse(BaseModel):
    """Response payload for sequence."""

    id: str
    name: str
    account_id: int

    @classmethod
    def from_entity(cls, sequence: Sequence) -> 'SequenceResponse':
        """Create response from MongoEngine document."""
        return cls(
            id=str(sequence.id),
            name=sequence.name,
            account_id=sequence.account_id
        )

    class Config:
        from_attributes = True
```

## Dependency Injection

The project uses `dependency-injector` for dependency management.

### Container Configuration

```python
from dependency_injector import containers, providers
from src.services.sequence_service import SequenceService

class ServicesContainer(containers.DeclarativeContainer):
    """Container for business services."""

    config = providers.Configuration()

    sequence_service = providers.Singleton(
        SequenceService,
        mongodb_host=config.mongodb_host,
    )

def build_container(container: containers.DynamicContainer) -> containers.DynamicContainer:
    """Build and wire dependency injection container."""
    services_container = ServicesContainer()
    services_container.config.from_dict({
        'mongodb_host': os.getenv('MONGODB_HOST', 'mongodb://localhost:27017'),
    })

    container.services_container = services_container
    return container
```

### Using Dependency Injection

```python
from dependency_injector.wiring import inject, Provide

@inject
def process_sequence(
    sequence_id: str,
    service: SequenceService = Provide["services_container.sequence_service"]
):
    """Process sequence using injected service."""
    return service.process(sequence_id)
```

## Testing Guidelines

### Test Organization

Tests are organized in `tests/` with clear separation:

```
tests/
├── unit/
│   ├── entity/
│   │   └── steps/
│   │       └── test_obfuscation.py
│   ├── dependency_injection/
│   └── payloads/
├── functional/
│   └── api/
│       └── test_endpoints.py
└── __init__.py
```

### Unit Test Example

```python
import unittest
from mongoengine import ValidationError

from src.entity.steps.obfuscation import ExclusionRule, MatchMode
from src.entity.steps.tolerance import Tolerance

class TestExclusionRule(unittest.TestCase):

    def test_default_values(self):
        """Test ExclusionRule default values."""
        exclusion_rule = ExclusionRule()

        self.assertIsNone(exclusion_rule.entity_group_pattern)
        self.assertIsNone(exclusion_rule.value_pattern)
        self.assertEqual(exclusion_rule.match_mode, MatchMode.STRICT)
        self.assertTrue(exclusion_rule.normalize_unicode)

    def test_custom_values(self):
        """Test ExclusionRule with custom values."""
        exclusion_rule = ExclusionRule(
            entity_group_pattern="*",
            value_pattern="*",
            match_mode=MatchMode.REGEX,
            normalize_unicode=False
        )

        self.assertEqual(exclusion_rule.entity_group_pattern, "*")
        self.assertEqual(exclusion_rule.match_mode, MatchMode.REGEX)
        self.assertFalse(exclusion_rule.normalize_unicode)

    def test_validate_tolerance(self):
        """Test tolerance validation."""
        self.assertIsNone(ExclusionRule.validate_tolerance(None))
        self.assertIsNone(ExclusionRule.validate_tolerance(True))
        self.assertIsNone(ExclusionRule.validate_tolerance(Tolerance()))

        with self.assertRaises(ValidationError):
            ExclusionRule.validate_tolerance("invalid_tolerance")

    def test_tolerance_field_validation(self):
        """Test tolerance field validation in document."""
        with self.assertRaises(ValidationError):
            ExclusionRule(tolerance="invalid_tolerance").validate()

        try:
            ExclusionRule(tolerance=True).validate()
        except ValidationError:
            self.fail("Validation raised unexpectedly!")
```

### Functional Test Example

```python
import pytest
from fastapi.testclient import TestClient
from mongomock import MongoClient

from src.api.app import create_app

@pytest.fixture
def client():
    """Create test client with mocked MongoDB."""
    app = create_app()
    with TestClient(app) as client:
        yield client

@pytest.fixture
def mongo_mock():
    """Mock MongoDB connection."""
    return MongoClient()

def test_create_sequence(client):
    """Test sequence creation endpoint."""
    response = client.post(
        "/api/v1/sequences",
        json={
            "name": "test-sequence",
            "account_id": 123,
            "language_detection": True
        }
    )

    assert response.status_code == 201
    data = response.json()
    assert data["name"] == "test-sequence"
    assert data["account_id"] == 123

def test_healthcheck(client):
    """Test healthcheck endpoint."""
    response = client.get("/healthcheck")

    assert response.status_code == 200
    assert response.json() == {"status": "healthy"}
```

### Pytest Configuration

```ini
# pytest.ini
[pytest]
asyncio_default_fixture_loop_scope = function
```

For functional tests with MongoDB, configure a test MongoDB instance (e.g., via Docker Compose or CI services):

```yaml
# Example CI service configuration
services:
  mongodb:
    image: mongo:6.0.15
    ports:
      - "27017:27017"

# Environment variable
MONGODB_HOST: mongodb://localhost:27017
```

### Best Practices

- Use `unittest.TestCase` for unit tests
- Use `pytest` fixtures for setup/teardown
- Use `mongomock` or test database for MongoDB tests
- Test one behavior per test method
- Use descriptive test names: `test_<what>_<scenario>`
- Use assertions from `unittest` (`assertEqual`, `assertIsNone`, etc.)
- Mock external dependencies in unit tests
- Use real services in functional tests

## OpenTelemetry Instrumentation

The project uses OpenTelemetry for observability.

### Configuration

```python
# In Dockerfile
ENV OTEL_PYTHON_EXCLUDED_URLS=healthcheck
ENV OTEL_SERVICE_NAME=my-service

# Command
CMD ["opentelemetry-instrument", "python", "src/main.py"]
```

### FastAPI Instrumentation

```python
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.httpx import HTTPXClientInstrumentor

# Instrument FastAPI
FastAPIInstrumentor.instrument_app(rest_app)

# Instrument httpx client
HTTPXClientInstrumentor().instrument()
```

### Environment Variables

- `OTEL_SERVICE_NAME`: Service name for traces
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP collector endpoint
- `OTEL_PYTHON_EXCLUDED_URLS`: URLs to exclude from tracing (e.g., healthcheck)

## Logging

Use Python's standard logging with structured output.

### Logging Configuration

```python
import logging
from pythonjsonlogger import jsonlogger

# Configure JSON logging
logger = logging.getLogger()
logHandler = logging.StreamHandler()
formatter = jsonlogger.JsonFormatter()
logHandler.setFormatter(formatter)
logger.addHandler(logHandler)
logger.setLevel(logging.INFO)
```

### Usage

```python
import logging

logger = logging.getLogger(__name__)

def process_request(sequence_id: str):
    logger.info("Processing sequence", extra={
        "sequence_id": sequence_id,
        "action": "process"
    })

    try:
        # Process
        logger.debug("Sequence loaded", extra={"sequence_id": sequence_id})
    except Exception as e:
        logger.error("Processing failed", extra={
            "sequence_id": sequence_id,
            "error": str(e)
        }, exc_info=True)
        raise
```

## Task Commands

The project uses Taskfile for common operations:

```yaml
# Taskfile.yml
tasks:
  lint:
    desc: Run ruff linter
    cmd: ruff check src/

  lint:fix:
    desc: Run ruff linter with auto-fix
    cmd: ruff check --fix src/

  format:
    desc: Run ruff formatter
    cmd: ruff format src/

  unused:
    desc: Find unused code with vulture
    cmd: vulture src/ --min-confidence 80

  dev:local:up:
    desc: Run local dev env
    cmds:
      - docker compose --profile infra --profile dependency up -d

  dev:docker:up:
    desc: Run docker dev env
    cmds:
      - docker compose --profile app --profile infra --profile dependency up -d
```

Usage:
```bash
task lint          # Run linter
task format        # Format code
task unused        # Find dead code
task dev:local:up  # Start local dev environment
```

## Error Handling

Use custom exceptions for domain errors:

```python
class SequenceError(Exception):
    """Base exception for sequence-related errors."""
    pass

class SequenceNotFoundError(SequenceError):
    """Raised when sequence is not found."""

    def __init__(self, sequence_id: str):
        self.sequence_id = sequence_id
        super().__init__(f"Sequence not found: {sequence_id}")

class ValidationError(SequenceError):
    """Raised when validation fails."""

    def __init__(self, field: str, message: str):
        self.field = field
        self.message = message
        super().__init__(f"Validation error in '{field}': {message}")

# Usage in FastAPI
@router.get("/sequences/{sequence_id}")
async def get_sequence(sequence_id: str):
    try:
        sequence = service.get(sequence_id)
        return SequenceResponse.from_entity(sequence)
    except SequenceNotFoundError:
        raise HTTPException(status_code=404, detail="Sequence not found")
    except ValidationError as e:
        raise HTTPException(status_code=400, detail=e.message)
```

## Configuration Management

Use environment variables for configuration:

```python
import os
from functools import lru_cache

class Settings:
    """Application settings from environment variables."""

    def __init__(self):
        self.app_env = os.getenv("APP_ENV", "development")
        self.mongodb_host = os.getenv("MONGODB_HOST", "mongodb://localhost:27017")
        self.otel_service_name = os.getenv("OTEL_SERVICE_NAME", "my-service")
        self.log_level = os.getenv("LOG_LEVEL", "INFO")

@lru_cache()
def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()

# Usage
settings = get_settings()
```
