# PHP Programming Guidelines

This document provides guidelines for writing PHP code in this project.

## Quick Reference

### Pre-commit Checklist
```bash
composer validate                                    # Validate composer.json
vendor/bin/grumphp run                              # Run GrumPHP tasks (PHP CS Fixer, PHPMD)
composer test-unit                                   # Run unit tests with coverage
composer test-func                                   # Run functional tests
composer audit                                       # Security audit
vendor/bin/infection                                 # Mutation testing (optional)
```

### GrumPHP Tasks
GrumPHP automatically runs pre-commit checks:
- **PHP CS Fixer**: Code formatting with @Symfony rules
- **PHPMD**: PHP Mess Detector for code quality
- **Git Blacklist**: Prevents committing debug code (var_dump, dd, die, etc.)

### Running Tests
```bash
# Unit tests with coverage
composer test-unit
# Or: XDEBUG_MODE=coverage bin/phpunit --testsuite=Unit --coverage-html coverage

# Functional tests
composer test-func
# Or: bin/phpunit --process-isolation --testsuite=Functional

# Specific test
bin/phpunit --filter testMethodName

# Mutation testing
composer test-all-infc
# Or: vendor/bin/infection --coverage=coverage --min-msi=99 --min-covered-msi=100
```

### Common Environment Variables
- `APP_ENV` - Application environment (dev, staging, prod)
- `APP_DEBUG` - Debug mode (0 or 1)
- `DATABASE_URL` - Database connection string (MariaDB/MySQL, PostgreSQL)
- `MESSENGER_TRANSPORT_DSN` - Symfony Messenger transport (AMQP/RabbitMQ)
- `REDIS_DSN` - Redis connection string (e.g., "redis://redis.external:6379")

## Guidelines for Agentic Contributors

When working as an AI agent on this project, you may need to examine source code from external dependencies to understand their implementation or debug issues.

### External Library Source Code Locations

- **Composer dependencies**: Look in `vendor/` directory which contains all installed packages
- **PSR autoloading**: Check `vendor/composer/autoload_*.php` files for autoload configuration

These locations contain the actual source code of external packages that your project depends on.

## Target Project Structure

These guidelines apply to Symfony projects that follow a business-logic-oriented architecture. The target projects are typically organized using PSR-4 autoloading with the following structure:

- **src/Business/** - Business logic organized by domain
  - Each domain contains: Manager, Action, Handler, Guard, Factory, Data (DTOs)
- **src/Controller/** - API Controllers (excluded from coverage)
- **src/Entity/** - Doctrine entities (excluded from coverage)
- **src/Repository/** - Doctrine repositories (excluded from coverage)
- **src/Form/** - Symfony forms (excluded from coverage)
- **src/Security/** - Security voters and authenticators
- **src/Command/** - Console commands
- **src/Migrations/** - Doctrine migrations (excluded from coverage)
- **tests/Unit/** - Unit tests (mirror src/ structure)
- **tests/Functional/** - Functional tests (API, commands)

**Note**: When working on a PHP project that includes these guidelines, you should apply the conventions described here.

## Code Style

The project uses Symfony 7.0 with PHP 8.3+ and follows PSR-12 coding standards with @Symfony rules.

### PHP Configuration

```php
// composer.json
{
    "require": {
        "php": ">=8.3",
        "symfony/framework-bundle": "^7.0"
    },
    "config": {
        "sort-packages": true,
        "allow-plugins": {
            "phpro/grumphp": true,
            "symfony/flex": true,
            "symfony/runtime": true
        }
    }
}
```

### Required Development Tools

The project uses these development tools (already configured):

- **PHPUnit 9.6.17**: Testing framework
- **GrumPHP**: Git hook manager for automated quality checks
- **PHP CS Fixer**: Code formatter with @Symfony rules
- **PHPMD**: PHP Mess Detector for code quality
- **Infection**: Mutation testing framework
- **Rector**: Automated refactoring tool
- **Hautelook Alice Bundle**: Fixture generation for tests
- **Doctrine Fixtures**: Database fixtures for testing

## Identify and Avoid Common Anti-Patterns

Before implementing your plan, check whether any common anti-patterns apply to your context. Refactor or plan around them where needed.

**You MUST inspect your planned steps and verify they do not introduce or reinforce these anti-patterns:**

* **Using global state** — Breaks testability and makes dependencies unclear. Use dependency injection instead.
* **Overusing static methods** — Makes testing difficult and creates tight coupling. Prefer instance methods with DI.
* **Ignoring type declarations** — Reduces type safety and IDE support. Always use strict types and declare return types.
* **Not using strict types** — Allows type coercion bugs. Always declare `declare(strict_types=1);` at the top of files.
* **Returning null instead of exceptions** — Makes error handling implicit and error-prone. Use typed exceptions for error cases.
* **Using arrays instead of objects** — Loses type safety and self-documentation. Create value objects and DTOs.
* **Not using readonly properties (PHP 8.1+)** — Allows unintended mutations. Use `readonly` for immutable data.
* **Ignoring named arguments (PHP 8.0+)** — Reduces code clarity for complex function calls. Use named arguments for clarity.
* **Not leveraging enums (PHP 8.1+)** — Uses magic strings/numbers instead of type-safe constants. Use enums for fixed sets of values.
* **Mixing business logic with infrastructure** — Creates tight coupling and testing difficulties. Follow clean architecture principles.

## Pre-commit Requirements

Before committing any changes to the repository, **GrumPHP automatically runs pre-commit checks**. You can also run them manually:

### 1. Run GrumPHP

GrumPHP runs automatically on `git commit`, or manually:

```bash
vendor/bin/grumphp run
```

GrumPHP configuration in `grumphp.yml`:

```yaml
grumphp:
    stop_on_failure: false
    tasks:
        phpcsfixer2:
            config: .php-cs-fixer.php
            using_cache: false
            config_contains_finder: true
        git_blacklist:
            keywords:
                - "die("
                - "var_dump("
                - "dump("
                - "dd("
            match_word: true
        phpmd:
            exclude: [vendor, translations, tests, templates, src/Factory]
            ruleset: ['cleancode', 'codesize']
```

### 2. PHP CS Fixer Configuration

The project uses @Symfony rules with custom adjustments in `.php-cs-fixer.php`:

```php
<?php

$finder = PhpCsFixer\Finder::create()
    ->in([__DIR__.'/src', __DIR__.'/tests']);

$config = new PhpCsFixer\Config();

return $config
    ->setUsingCache(false)
    ->setFinder($finder)
    ->setRules([
        '@Symfony'                => true,
        'binary_operator_spaces'  => ['operators' => ['=' => 'align', '=>' => 'align']],
        'psr_autoloading'         => false,
        'ordered_imports'         => true,
        'array_syntax'            => ['syntax' => 'short'],
        'global_namespace_import' => ['import_classes' => true],
    ]);
```

### 3. Run Tests Before Commit

```bash
# Unit tests with coverage
composer test-unit

# Functional tests
composer test-func
```

### 4. Security Audit

Check for package vulnerabilities:

```bash
composer audit
```

## Logging and Instrumentation

The project uses PSR-3 compliant logging with structured log contexts. Use Monolog or similar PSR-3 logger.

### Using PSR-3 Logging

```php
<?php

declare(strict_types=1);

namespace App\Service;

use Psr\Log\LoggerInterface;

final readonly class MessageProcessor
{
    public function __construct(
        private LoggerInterface $logger,
    ) {
    }

    public function process(Message $message): void
    {
        $this->logger->info('Processing message', [
            'message_id' => $message->id,
            'message_type' => $message->type,
        ]);

        try {
            // Process message
            $this->logger->debug('Message processed successfully', [
                'message_id' => $message->id,
            ]);
        } catch (\Throwable $e) {
            $this->logger->error('Failed to process message', [
                'message_id' => $message->id,
                'error' => $e->getMessage(),
                'exception' => $e,
            ]);
            throw $e;
        }
    }
}
```

### Log Levels

Use appropriate log levels based on the importance and frequency of the message:

- **emergency()** - System is unusable (e.g., database completely down)
- **alert()** - Action must be taken immediately (e.g., entire website down)
- **critical()** - Critical conditions (e.g., application component unavailable)
- **error()** - Runtime errors that don't require immediate action
- **warning()** - Exceptional occurrences that are not errors
- **notice()** - Normal but significant events
- **info()** - Interesting events (e.g., user logged in, message processed)
- **debug()** - Detailed debug information

### Best Practices

- Use structured logging with context arrays
- Never log sensitive data (passwords, tokens, PII)
- Use appropriate log levels
- Include relevant context (IDs, types, actions)
- Log exceptions with full context

## Error Handling

PHP 8.2+ provides excellent exception handling capabilities. Use typed exceptions and avoid error suppression.

### Defining Exception Types

```php
<?php

declare(strict_types=1);

namespace App\Exception;

final class MessageProcessingException extends \RuntimeException
{
    public static function deserializationFailed(string $reason): self
    {
        return new self("Failed to deserialize message: {$reason}");
    }

    public static function publishFailed(string $topic, \Throwable $previous): self
    {
        return new self("Failed to publish message to topic '{$topic}'", 0, $previous);
    }
}

final class ValidationException extends \InvalidArgumentException
{
    public static function missingField(string $field): self
    {
        return new self("Field '{$field}' is required");
    }

    public static function invalidFormat(string $field, string $reason): self
    {
        return new self("Invalid format for '{$field}': {$reason}");
    }

    public static function outOfRange(int $value, int $min, int $max): self
    {
        return new self("Value {$value} is out of range [{$min}, {$max}]");
    }
}
```

### Error Propagation

Use typed exceptions and let them bubble up:

```php
<?php

declare(strict_types=1);

namespace App\Service;

use App\Exception\ConfigurationException;

final readonly class ConfigLoader
{
    /**
     * @throws ConfigurationException
     */
    public function load(string $path): Configuration
    {
        if (!file_exists($path)) {
            throw ConfigurationException::fileNotFound($path);
        }

        $content = file_get_contents($path);
        if ($content === false) {
            throw ConfigurationException::cannotRead($path);
        }

        try {
            $data = json_decode($content, true, 512, JSON_THROW_ON_ERROR);
        } catch (\JsonException $e) {
            throw ConfigurationException::invalidJson($path, $e);
        }

        return Configuration::fromArray($data);
    }
}
```

### Best Practices

- Always declare `@throws` in docblocks
- Use specific exception types, not generic \Exception
- Include context in exception messages
- Chain exceptions using previous parameter
- Never use `@` error suppression operator
- Use try-catch only where you can handle the exception

## Environment Variables

The project uses environment variables for configuration. Use `.env` files for local development.

### Reading Environment Variables

```php
<?php

declare(strict_types=1);

namespace App\Config;

final readonly class AppConfig
{
    public function __construct(
        public string $environment,
        public string $databaseUrl,
        public string $redisUrl,
        public array $kafkaBrokers,
    ) {
    }

    public static function fromEnv(): self
    {
        $env = $_ENV['APP_ENV'] ?? throw new \RuntimeException('APP_ENV must be set');
        $databaseUrl = $_ENV['DATABASE_URL'] ?? throw new \RuntimeException('DATABASE_URL must be set');
        $redisUrl = $_ENV['REDIS_URL'] ?? 'redis://localhost:6379';
        $kafkaBrokers = explode(',', $_ENV['KAFKA_BROKERS'] ?? 'localhost:9092');

        return new self(
            environment: $env,
            databaseUrl: $databaseUrl,
            redisUrl: $redisUrl,
            kafkaBrokers: $kafkaBrokers,
        );
    }
}
```

### Using vlucas/phpdotenv

```php
<?php

declare(strict_types=1);

use Dotenv\Dotenv;

$dotenv = Dotenv::createImmutable(__DIR__);
$dotenv->load();

// Required variables
$dotenv->required(['APP_ENV', 'DATABASE_URL']);

// Optional variables with validation
$dotenv->ifPresent('REDIS_URL')->notEmpty();
```

## Testing Guidelines

### Test Organization

The project follows PHPUnit testing conventions with clear separation between unit tests and functional tests.

#### Unit Tests

Unit tests must be placed in `tests/Unit/` directory, mirroring the `src/` structure:

```
tests/
├── Unit/
│   └── Business/
│       ├── Account/
│       │   └── Manager/
│       │       └── AccountManagerTest.php
│       └── User/
│           └── Manager/
│               └── UserManagerTest.php
└── Functional/
    ├── Controller/
    │   └── UserControllerTest.php
    └── Command/
        └── CleanAccountCommandTest.php
```

Unit tests focus on testing business logic in isolation with mocked dependencies.

**Example Unit Test:**

```php
<?php

namespace App\Tests\Unit\Business\Order\Manager;

use App\Business\Order\Manager\OrderManager;
use App\Entity\Order\Order;
use App\Repository\Order\OrderRepository;
use PHPUnit\Framework\MockObject\MockObject;
use PHPUnit\Framework\TestCase;
use Symfony\Contracts\HttpClient\HttpClientInterface;
use Symfony\Contracts\HttpClient\ResponseInterface;

class OrderManagerTest extends TestCase
{
    private string $apiBaseUrl;
    private MockObject $httpClient;
    private MockObject $order;
    private OrderManager $orderManager;
    private MockObject $orderRepository;

    protected function setUp(): void
    {
        $this->apiBaseUrl      = 'http://api.test';
        $this->httpClient      = $this->createMock(HttpClientInterface::class);
        $this->order           = $this->createMock(Order::class);
        $this->orderRepository = $this->createMock(OrderRepository::class);

        $this->orderManager = new OrderManager(
            $this->orderRepository,
            $this->httpClient,
            $this->apiBaseUrl,
        );
    }

    public function testGetOrdersShouldSuccess(): void
    {
        $orders = [
            ['id' => 1, 'status' => 'pending'],
            ['id' => 2, 'status' => 'completed'],
        ];

        $response = $this->createMock(ResponseInterface::class);
        $response
            ->expects($this->once())
            ->method('toArray')
            ->willReturn($orders);

        $this->httpClient
            ->expects($this->once())
            ->method('request')
            ->with('GET', $this->apiBaseUrl . '/orders')
            ->willReturn($response);

        $result = $this->orderManager->getOrders();

        self::assertEquals($orders, $result);
    }
}
```

**Key patterns:**
- Use `setUp()` to initialize mocks and dependencies
- Create mocks with `$this->createMock(ClassName::class)`
- Use `expects()` and `willReturn()` for mock expectations
- Test method names use camelCase (not snake_case)

#### Functional Tests

Functional tests must be placed in `tests/Functional/` directory and extend `AbstractWebTestCase`:

```php
<?php

declare(strict_types=1);

namespace Tests\Integration\Infrastructure;

use App\Infrastructure\Database\UserRepository;
use PHPUnit\Framework\TestCase;

final class UserRepositoryTest extends TestCase
{
    private UserRepository $repository;

    protected function setUp(): void
    {
        parent::setUp();

        // Set up test database
        $this->repository = new UserRepository(
            $this->createTestDatabaseConnection()
        );
    }

    public function testSaveAndRetrieveUser(): void
    {
        $user = new User(
            id: 'user-123',
            email: 'test@example.com',
            name: 'Test User',
        );

        $this->repository->save($user);
        $retrieved = $this->repository->findById('user-123');

        self::assertEquals($user, $retrieved);
    }

    protected function tearDown(): void
    {
        // Clean up test data
        $this->repository->deleteAll();
        parent::tearDown();
    }
}
```

### Running Tests

PHPUnit configuration in `phpunit.xml.dist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="https://schema.phpunit.de/9.5/phpunit.xsd"
         colors="true"
         bootstrap="tests/bootstrap.php"
         processIsolation="false"
         stopOnFailure="false"
         defaultTestSuite="Unit">
  <coverage>
    <include>
      <directory>./src/</directory>
    </include>
    <exclude>
      <directory>src/Migrations</directory>
      <directory>src/Controller</directory>
      <directory>src/Security/Voter</directory>
      <directory>src/Factory</directory>
      <directory>src/DataFixtures</directory>
      <directory>src/Entity</directory>
      <directory>src/Repository</directory>
      <directory>src/Form</directory>
    </exclude>
  </coverage>
  <php>
    <server name="SYMFONY_PHPUNIT_VERSION" value="9.6.17"/>
    <server name="KERNEL_CLASS" value="App\Kernel" />
  </php>
  <testsuites>
    <testsuite name="Unit">
      <directory>tests/Unit</directory>
    </testsuite>
    <testsuite name="Functional">
      <directory>tests/Functional</directory>
    </testsuite>
  </testsuites>
</phpunit>
```

**Coverage exclusions:**
- Controllers, Entities, Repositories, Forms are excluded (framework/ORM code)
- Migrations and Factories are excluded
- Focus coverage on business logic in `src/Business/`

### Test Best Practices

When adding new tests:

1. Use descriptive test method names (testMethodName or test_method_name)
2. Follow Arrange-Act-Assert pattern
3. Use type declarations for all parameters and return types
4. Use `self::assert*()` instead of `$this->assert*()`
5. Test one behavior per test method
6. Use data providers for testing multiple scenarios
7. Mock external dependencies in unit tests
8. Use real infrastructure in integration tests

**Example with Data Provider:**

```php
<?php

declare(strict_types=1);

namespace Tests\Unit\Domain;

use App\Domain\EmailValidator;
use PHPUnit\Framework\TestCase;

final class EmailValidatorTest extends TestCase
{
    /**
     * @dataProvider validEmailProvider
     */
    public function testValidEmails(string $email): void
    {
        $validator = new EmailValidator();
        self::assertTrue($validator->isValid($email));
    }

    /**
     * @return array<string, array{string}>
     */
    public static function validEmailProvider(): array
    {
        return [
            'simple' => ['user@example.com'],
            'with plus' => ['user+tag@example.com'],
            'subdomain' => ['user@mail.example.com'],
        ];
    }

    /**
     * @dataProvider invalidEmailProvider
     */
    public function testInvalidEmails(string $email): void
    {
        $validator = new EmailValidator();
        self::assertFalse($validator->isValid($email));
    }

    /**
     * @return array<string, array{string}>
     */
    public static function invalidEmailProvider(): array
    {
        return [
            'no at sign' => ['userexample.com'],
            'no domain' => ['user@'],
            'no user' => ['@example.com'],
        ];
    }
}
```

## Modern PHP Features

### Readonly Properties (PHP 8.1+)

Use readonly properties for immutable data:

```php
<?php

declare(strict_types=1);

namespace App\Domain;

final readonly class User
{
    public function __construct(
        public string $id,
        public string $email,
        public string $name,
    ) {
    }
}
```

### Enums (PHP 8.1+)

Use enums for fixed sets of values:

```php
<?php

declare(strict_types=1);

namespace App\Domain;

enum UserStatus: string
{
    case Active = 'active';
    case Inactive = 'inactive';
    case Suspended = 'suspended';

    public function isActive(): bool
    {
        return $this === self::Active;
    }
}

enum MessageType
{
    case UserCreated;
    case UserUpdated;
    case UserDeleted;
}
```

### Named Arguments (PHP 8.0+)

Use named arguments for clarity:

```php
<?php

declare(strict_types=1);

// Good - clear and readable
$user = new User(
    id: 'user-123',
    email: 'test@example.com',
    name: 'Test User',
);

// Good - skip optional parameters
$config = new DatabaseConfig(
    host: 'localhost',
    database: 'myapp',
    username: 'root',
    // password is optional
);
```

### Union Types (PHP 8.0+)

Use union types for flexible parameters:

```php
<?php

declare(strict_types=1);

namespace App\Service;

final readonly class IdGenerator
{
    public function generate(string|int $seed): string
    {
        return hash('sha256', (string) $seed);
    }
}
```

### Constructor Property Promotion (PHP 8.0+)

Use constructor property promotion to reduce boilerplate:

```php
<?php

declare(strict_types=1);

namespace App\Domain;

// Modern approach with property promotion
final readonly class Product
{
    public function __construct(
        public string $id,
        public string $name,
        public float $price,
        public bool $available = true,
    ) {
    }
}
```

## Dependency Injection

Use constructor injection for all dependencies:

```php
<?php

declare(strict_types=1);

namespace App\Application;

use App\Domain\UserRepositoryInterface;
use App\Infrastructure\Messaging\MessagePublisherInterface;
use Psr\Log\LoggerInterface;

final readonly class UserService
{
    public function __construct(
        private UserRepositoryInterface $userRepository,
        private MessagePublisherInterface $messagePublisher,
        private LoggerInterface $logger,
    ) {
    }

    public function createUser(string $email, string $name): User
    {
        $user = new User(
            id: $this->generateId(),
            email: $email,
            name: $name,
        );

        $this->userRepository->save($user);

        $this->messagePublisher->publish(
            new UserCreatedMessage($user->id, $user->email)
        );

        $this->logger->info('User created', [
            'user_id' => $user->id,
            'email' => $user->email,
        ]);

        return $user;
    }

    private function generateId(): string
    {
        return uniqid('user-', true);
    }
}
```

## Type Safety

Always use strict types and declare all types:

```php
<?php

declare(strict_types=1);

namespace App\Domain;

final readonly class Money
{
    public function __construct(
        public float $amount,
        public string $currency,
    ) {
        if ($amount < 0) {
            throw new \InvalidArgumentException('Amount cannot be negative');
        }

        if (!in_array($currency, ['USD', 'EUR', 'GBP'], true)) {
            throw new \InvalidArgumentException("Invalid currency: {$currency}");
        }
    }

    public function add(self $other): self
    {
        if ($this->currency !== $other->currency) {
            throw new \InvalidArgumentException('Cannot add different currencies');
        }

        return new self(
            amount: $this->amount + $other->amount,
            currency: $this->currency,
        );
    }

    public function multiply(float $factor): self
    {
        return new self(
            amount: $this->amount * $factor,
            currency: $this->currency,
        );
    }
}
```
