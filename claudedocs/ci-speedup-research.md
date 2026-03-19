# CI Pipeline Speedup Research: Rust + Nextest + Testcontainers on GitHub Actions

**Date**: 2026-03-10
**Context**: Offrii backend -- 567 integration tests across 14 test files, ~620s runtime, using testcontainers (PostgreSQL 16-alpine, Redis 7), cargo-nextest, cargo-llvm-cov, GitHub Actions on `ubuntu-latest`.

---

## Executive Summary

Your biggest bottleneck is that **every single test spins up its own PostgreSQL + Redis container pair** via `TestApp::new()`. With 567 tests and nextest running each test in a separate process, that means up to hundreds of container startups. The single highest-ROI change is reducing container overhead. The second is splitting test execution across multiple CI jobs via nextest partitioning.

**Estimated combined impact**: 620s down to ~120-180s (70-80% reduction).

**ROI-ranked strategy list**:

| Priority | Strategy | Estimated Savings | Effort |
|----------|----------|-------------------|--------|
| 1 | Nextest partitioning (3-4 shards) | 60-75% of test time | Low |
| 2 | sccache for compilation | 50-70% of build time | Low |
| 3 | CI-specific Cargo profile | 20-40% of build time | Low |
| 4 | Nextest test-threads tuning | 10-20% of test time | Low |
| 5 | Nextest test groups for heavy tests | 10-15% of test time | Medium |
| 6 | GitHub Actions service containers (replace testcontainers) | 30-50% of test time | High |
| 7 | Larger runners | 20-40% of total time | Cost |
| 8 | Test architecture: shared fixtures | 40-60% of test time | High |

---

## 1. Nextest-Specific Optimizations

### 1.1 Partitioning Across CI Jobs (HIGH ROI)

Nextest has first-class support for splitting tests across multiple jobs. This is the single easiest way to cut wall-clock time.

**How it works**: Build once, archive the test binaries, then fan out to N parallel jobs each running a slice of the test suite.

Create `.config/nextest.toml` in your `backend/` workspace root:

```toml
# backend/.config/nextest.toml

[store]
dir = "target/nextest"

# ---------- default (local dev) ----------
[profile.default]
test-threads = "num-cpus"
fail-fast = true
slow-timeout = { period = "60s", terminate-after = 3 }
status-level = "retry"
failure-output = "immediate-final"

# ---------- CI profile ----------
[profile.ci]
test-threads = 4                         # GitHub Actions standard runners have 4 vCPUs
retries = 1                              # retry flaky tests once
fail-fast = { max-fail = 5 }             # don't bail on first failure in CI
slow-timeout = { period = "120s", terminate-after = 2 }
failure-output = "immediate-final"
final-status-level = "fail"

# ---------- per-test overrides ----------
# Mark community_wish_tests as heavy (they have 129 tests and likely
# dominate the test run)
[[profile.ci.overrides]]
filter = 'binary_id(rest-api::community_wish_tests)'
threads-required = 2

# Migration tests touch schema -- serialize them
[[profile.ci.overrides]]
filter = 'binary_id(rest-api::migration_tests)'
test-group = 'serial-db'

[test-groups]
serial-db = { max-threads = 1 }
```

### 1.2 Partitioned CI Workflow (Build Once, Test Many)

Replace your current single backend job with a build+partition pattern:

```yaml
# .github/workflows/ci.yml  (backend section)

  backend-build:
    needs: detect-changes
    if: needs.detect-changes.outputs.backend == 'true'
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy, llvm-tools-preview

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.7
      - name: Configure sccache env
        run: |
          echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
          echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target

      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-sort,cargo-machete,cargo-nextest,cargo-llvm-cov

      # Fast checks first (fail fast)
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Check Cargo.toml sorting
        run: cargo sort --workspace --check
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Check unused dependencies
        run: cargo machete

      # Build and archive test binaries (ONE compilation)
      - name: Build and archive tests
        run: cargo nextest archive --workspace --archive-file target/nextest-archive.tar.zst

      - name: Upload test archive
        uses: actions/upload-artifact@v4
        with:
          name: nextest-archive
          path: backend/target/nextest-archive.tar.zst
          retention-days: 1

  backend-test:
    needs: backend-build
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        partition: [1, 2, 3]
    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v6
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Download test archive
        uses: actions/download-artifact@v4
        with:
          name: nextest-archive
          path: backend/target/

      - name: Pre-pull testcontainer images
        run: |
          docker pull postgres:16-alpine &
          docker pull redis:7 &
          wait

      - name: Run tests (partition ${{ matrix.partition }}/3)
        run: |
          cargo nextest run \
            --archive-file target/nextest-archive.tar.zst \
            --profile ci \
            --partition slice:${{ matrix.partition }}/3
        env:
          NEXTEST_PROFILE: ci
```

**Key points**:
- `slice:` partitioning distributes tests round-robin across shards, providing even distribution.
- 3 partitions is a good starting point for 567 tests. Each shard gets ~189 tests.
- The build phase compiles once; test shards only need nextest (not even `cargo`).
- `fail-fast: false` on the matrix lets all shards complete even if one fails.

### 1.3 Coverage Integration with Partitioning

If you need to keep `cargo-llvm-cov`, it does not directly support the archive workflow. Two options:

**Option A**: Keep coverage in a separate dedicated job (simpler):
```yaml
  backend-coverage:
    needs: backend-build
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest,cargo-llvm-cov
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target
      - name: Pre-pull images
        run: |
          docker pull postgres:16-alpine &
          docker pull redis:7 &
          wait
      - name: Coverage
        run: cargo llvm-cov nextest --workspace --profile ci --fail-under-lines 75 --ignore-filename-regex "(main|migrate)\.rs$"
```

**Option B**: Run coverage only on merge to master, run partitioned tests on PRs. This is the most practical approach -- PRs get fast feedback, main branch gets coverage gating.

### 1.4 Test-Threads Tuning

Standard GitHub Actions runners have **4 vCPUs and 16 GB RAM**. With testcontainers, each test process spawns Docker containers, so `num-cpus` (4) is actually reasonable. However, if container startup is the bottleneck, increasing threads won't help -- it will just queue more container starts.

If you move to service containers (see section 3), you can safely increase:
```toml
[profile.ci]
test-threads = 8    # or even 12, since tests become lightweight
```

---

## 2. GitHub Actions Optimizations

### 2.1 sccache (HIGH ROI)

`sccache` caches compiled Rust artifacts at the `rustc` invocation level. Unlike `Swatinem/rust-cache` (which caches the `target/` directory), sccache works incrementally and starts builds immediately without waiting for a full cache download.

**Use both together** -- they complement each other:

```yaml
      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.7

      - name: Configure sccache
        run: |
          echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
          echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target
          cache-targets: true
```

**Expected improvement**: First build populates the cache. Subsequent builds (with only source changes) hit 70-90% cache rate, reducing compilation from minutes to under 60 seconds for incremental changes.

### 2.2 Larger Runners (COST vs SPEED tradeoff)

If your GitHub plan supports it (Team or Enterprise Cloud):

```yaml
    runs-on: ubuntu-latest-8-core   # 8 vCPU, 32 GB RAM
    # or
    runs-on: ubuntu-latest-16-core  # 16 vCPU, 64 GB RAM
```

With 8 cores, nextest can run 8 tests in parallel. With 16 cores, 16 tests. This directly multiplies throughput. Cost is typically 2x/4x of standard runners.

**Recommendation**: Start with standard runners + partitioning. Only upgrade to larger runners if you need sub-3-minute builds.

### 2.3 Concurrency Control

You already have this, which is good:
```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

This prevents wasted resources on superseded pushes.

### 2.4 Docker Image Pre-pulling

You already do this correctly with background pulls:
```yaml
      - name: Pre-pull testcontainers images
        run: |
          docker pull postgres:16-alpine &
          docker pull redis:7 &
          wait
```

The `&` + `wait` pattern runs both pulls in parallel. This saves 5-10 seconds per shard.

---

## 3. Testcontainer Optimizations

### 3.1 The Core Problem in Your Architecture

Looking at your `tests/common/mod.rs`, every test creates a full `TestApp::new()` which starts **a fresh PostgreSQL container + Redis container + runs all migrations**. With nextest running each test as a separate process, that is potentially 567 container-start-migration cycles.

This is your single biggest source of slowness. Each container startup + migration takes ~2-5 seconds. Across hundreds of tests, this dominates.

### 3.2 Option A: GitHub Actions Service Containers (RECOMMENDED for CI)

Instead of testcontainers, use GitHub Actions native service containers. The database starts once for the entire job and all tests connect to it. This eliminates hundreds of container startups.

```yaml
  backend-test:
    needs: backend-build
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        partition: [1, 2, 3]

    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: postgres
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 5s
          --health-timeout 5s
          --health-retries 5

    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v6
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Download test archive
        uses: actions/download-artifact@v4
        with:
          name: nextest-archive
          path: backend/target/

      - name: Run migrations
        run: cargo run --bin migrate
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/postgres

      - name: Run tests
        run: |
          cargo nextest run \
            --archive-file target/nextest-archive.tar.zst \
            --profile ci \
            --partition slice:${{ matrix.partition }}/3
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/postgres
          REDIS_URL: redis://localhost:6379
```

**This requires refactoring `TestApp::new()`** to accept a connection URL from environment variables instead of spinning up containers. See section 5.2.

### 3.3 Option B: Per-Binary Container Sharing (Lower effort, moderate payoff)

Since nextest runs each test binary as a separate process but groups all `#[test]` functions within one binary, you get **one container per test file** (14 containers total instead of 567).

Wait -- this is actually how nextest already works! Each test file is a separate binary, and tests within that binary share the same process. So `TestApp::new()` in `community_wish_tests.rs` creates containers once for that binary, and all 129 tests in it share those containers.

But there is a subtlety: **nextest runs each test in its own process invocation of the binary**, unlike `cargo test` which runs all tests within one binary in a single process. This means `TestApp::new()` IS called 567 times.

This confirms that the testcontainer overhead is the dominant factor.

### 3.4 Option C: Reusable Containers Feature (Moderate effort)

The Rust `testcontainers` crate supports a `reusable-containers` feature flag:

```toml
# In Cargo.toml [dev-dependencies]
testcontainers = { version = "0.27", features = ["reusable-containers"] }
```

This allows containers to persist across test runs. However, with nextest's process-per-test model, this is less effective because each process would still need to connect to the Docker API to find/create the container. The overhead reduction is moderate (skip container creation but still incur Docker API round-trip).

### 3.5 Pre-pull in CI (Already implemented)

You already do this. Keep it.

---

## 4. Rust Compilation Optimizations

### 4.1 CI-Specific Cargo Profile (HIGH ROI)

Add a custom CI profile that prioritizes compilation speed over everything else:

```toml
# backend/Cargo.toml

# CI profile: optimize for fast compilation, not runtime speed
[profile.ci]
inherits = "dev"
opt-level = 0
debug = 0              # no debug info = significantly faster
strip = "debuginfo"
incremental = false     # incremental adds overhead in CI (no warm cache)
codegen-units = 256     # maximum parallelism during codegen

[profile.ci.build-override]
opt-level = 0
codegen-units = 256
```

Then in CI:
```yaml
      - name: Build tests
        run: cargo nextest archive --workspace --profile ci --archive-file target/nextest-archive.tar.zst
        env:
          CARGO_PROFILE: ci
```

**Key decisions explained**:
- `debug = 0`: Skipping debuginfo generation can save 20-40% of compilation time. You lose backtraces in test failures, but the test output itself (assertion messages) is still available.
- `incremental = false`: `Swatinem/rust-cache` already sets `CARGO_INCREMENTAL=0`. Incremental compilation adds I/O overhead and is only beneficial for warm local caches.
- `codegen-units = 256`: Maximum parallelism during code generation. Produces slower code but compiles faster -- fine for tests.

### 4.2 Faster Linker (HIGH ROI)

Install `lld` or `mold` for significantly faster linking:

```yaml
      - name: Install fast linker
        run: sudo apt-get update && sudo apt-get install -y lld

      # Then set RUSTFLAGS
      - name: Build
        run: cargo nextest archive --workspace --archive-file target/nextest-archive.tar.zst
        env:
          RUSTFLAGS: "-C link-arg=-fuse-ld=lld"
```

Or create `backend/.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Expected improvement**: 30-50% faster linking. Linking is typically 10-30% of total compile time for Rust projects, so this saves 3-15% overall.

### 4.3 sccache (covered in section 2.1)

sccache is the single most impactful compilation optimization for CI. The first build populates the cache; subsequent builds with only source changes recompile just your crate (not dependencies).

### 4.4 Disable Unused Components for Test Builds

If you're building with `--all-targets`, Clippy and tests compile different things. Make sure your test build doesn't also build release artifacts:

```yaml
      # Build only test binaries, not release
      - name: Build and archive tests
        run: cargo nextest archive --workspace --archive-file target/nextest-archive.tar.zst
```

`cargo nextest archive` already only compiles test binaries, which is correct.

---

## 5. Test Architecture Changes

### 5.1 Current Architecture Analysis

Your `TestApp::new()` does this per test:
1. Start PostgreSQL container (~1-3s)
2. Start Redis container (~1-2s)
3. Connect to PostgreSQL
4. Run all migrations (~0.5-1s)
5. Wire up all 20+ repositories and services
6. Build the full router

Steps 1-4 are the expensive ones. Steps 5-6 are fast (in-memory operations).

### 5.2 Refactored TestApp: Environment-Based Connection (RECOMMENDED)

Modify `TestApp::new()` to optionally use pre-existing databases:

```rust
// tests/common/mod.rs

use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for creating unique database names per test
static DB_COUNTER: AtomicU32 = AtomicU32::new(0);

#[allow(dead_code)]
pub struct TestApp {
    // Make containers optional -- they're None when using external services
    _pg_container: Option<ContainerAsync<Postgres>>,
    _redis_container: Option<ContainerAsync<Redis>>,
    pub router: Router,
    pub db: PgPool,
    pub redis: redis::Client,
    pub last_reset_code: Arc<StdMutex<Option<String>>>,
}

#[allow(dead_code)]
impl TestApp {
    pub async fn new() -> Self {
        // Check for external database (CI service containers)
        let (pg_url, pg_container) = if let Ok(url) = std::env::var("DATABASE_URL") {
            // CI mode: use service container, create isolated database per test
            let db_id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db_name = format!("test_db_{db_id}_{}", std::process::id());

            // Connect to default database to create the test database
            let admin_pool = PgPool::connect(&url).await.unwrap();
            sqlx::query(&format!("CREATE DATABASE \"{test_db_name}\""))
                .execute(&admin_pool)
                .await
                .unwrap();
            admin_pool.close().await;

            // Build URL for the test database
            let test_url = format!(
                "{}/{}",
                url.rsplitn(2, '/').last().unwrap(),
                test_db_name
            );
            (test_url, None)
        } else {
            // Local mode: spin up testcontainer
            let container = Postgres::default()
                .with_tag("16-alpine")
                .start()
                .await
                .unwrap();
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(5432).await.unwrap();
            let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
            (url, Some(container))
        };

        let (redis_url, redis_container) = if let Ok(url) = std::env::var("REDIS_URL") {
            (url, None)
        } else {
            let container = Redis::default().with_tag("7").start().await.unwrap();
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(6379).await.unwrap();
            (format!("redis://{host}:{port}"), Some(container))
        };

        let db = create_pg_pool(&pg_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();

        let redis = create_redis_client(&redis_url).unwrap();

        // ... rest of the DI wiring stays exactly the same ...
    }
}
```

**Benefits**:
- Locally, developers get the same testcontainer experience (zero config)
- In CI, all tests connect to a single pre-started PostgreSQL/Redis
- Each test gets its own database (isolation preserved via `CREATE DATABASE`)
- Migration runs per test database (~0.5s) instead of container startup (~3s)

**Note on isolation**: Each test creates a unique database name using `process::id()` + atomic counter, so parallel tests never interfere. This is the standard pattern used by frameworks like Rails and Django.

### 5.3 Alternative: Transaction-Based Isolation (Maximum Speed)

For even faster tests, wrap each test in a transaction that rolls back:

```rust
impl TestApp {
    /// Run a test inside a transaction that rolls back.
    /// This is the fastest approach but requires the test to only
    /// use the provided connection (not create its own transactions).
    pub async fn with_transaction<F, Fut>(&self, test: F)
    where
        F: FnOnce(&PgPool) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut tx = self.db.begin().await.unwrap();
        // ... run test with transactional connection ...
        tx.rollback().await.unwrap();
    }
}
```

This is faster but requires that tests don't commit their own transactions, which may conflict with your service layer.

### 5.4 Reduce Per-Test DI Overhead

Your current `TestApp::new()` wires up ALL repositories and services for EVERY test. Most tests only need a subset. Consider a builder pattern:

```rust
// Future optimization: TestAppBuilder that only wires needed services
// This reduces allocation overhead but is lower priority than
// container optimization.
```

This is lower priority since the DI wiring is fast (all in-memory). Focus on container overhead first.

---

## 6. Parallelization Strategies

### 6.1 Recommended Shard Count

For 567 tests with current architecture (testcontainers per test):

| Shards | Est. Time per Shard | Total Wall Clock | Notes |
|--------|---------------------|------------------|-------|
| 1 | 620s | 620s | Current |
| 2 | ~320s | ~320s | Minimal overhead |
| 3 | ~220s | ~220s | Good balance |
| 4 | ~170s | ~170s | Diminishing returns |
| 6 | ~120s | ~120s | Worth it if combined with service containers |

With service containers (no testcontainer overhead), tests run ~2-5x faster individually, so:

| Shards | Est. Time per Shard | Total Wall Clock |
|--------|---------------------|------------------|
| 2 | ~90s | ~90s |
| 3 | ~65s | ~65s |
| 4 | ~50s | ~50s |

### 6.2 Optimal Partitioning Strategy

Use `slice:` (not `count:` which is deprecated):

```bash
cargo nextest run --partition slice:1/3  # 189 tests
cargo nextest run --partition slice:2/3  # 189 tests
cargo nextest run --partition slice:3/3  # 189 tests
```

Sliced partitioning distributes tests round-robin across all binaries, giving better balance than per-binary splitting.

### 6.3 Test Distribution Concern

Your test distribution is uneven:
- `community_wish_tests.rs`: 129 tests (23%)
- `item_tests.rs`: 72 tests (13%)
- `friend_tests.rs`: 68 tests (12%)
- `circle_tests.rs`: 66 tests (12%)
- `auth_tests.rs`: 65 tests (11%)

Nextest's slice partitioning handles this well because it assigns individual tests round-robin, not entire binaries. So even the 129-test file gets split across shards.

---

## 7. Complete Recommended CI Configuration

### Phase 1: Quick Wins (implement first)

1. Add `.config/nextest.toml` with CI profile (section 1.1)
2. Add sccache to build step (section 2.1)
3. Add `lld` linker (section 4.2)
4. Split into build + 3 partitioned test jobs (section 1.2)

**Expected result**: ~620s down to ~200s (each shard ~200s, running in parallel)

### Phase 2: Architecture Change (implement second)

5. Refactor `TestApp::new()` to support env-based connections (section 5.2)
6. Use GitHub Actions service containers (section 3.2)

**Expected result**: ~200s per shard down to ~60-80s per shard, total wall clock ~80s

### Phase 3: Fine Tuning (implement if needed)

7. Larger runners (section 2.2)
8. Test groups for heavy tests (section 1.1)
9. Increase to 4 shards if needed
10. Transaction-based test isolation (section 5.3)

### Complete `backend/.config/nextest.toml`

```toml
[store]
dir = "target/nextest"

# ---------- Local development ----------
[profile.default]
test-threads = "num-cpus"
fail-fast = true
slow-timeout = { period = "60s", terminate-after = 3 }
failure-output = "immediate-final"

# ---------- CI ----------
[profile.ci]
test-threads = 4
retries = 1
fail-fast = { max-fail = 5 }
slow-timeout = { period = "120s", terminate-after = 2 }
failure-output = "immediate-final"
final-status-level = "fail"

# Heavy test file -- requires more resources per test
[[profile.ci.overrides]]
filter = 'binary_id(rest-api::community_wish_tests)'
threads-required = 2

# Migration tests modify schema -- serialize
[[profile.ci.overrides]]
filter = 'binary_id(rest-api::migration_tests)'
test-group = 'serial-db'

[test-groups]
serial-db = { max-threads = 1 }
```

### Complete `backend/.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Complete Cargo.toml Profile Additions

```toml
# Add to backend/Cargo.toml

# Fast CI compilation -- no debug info, max codegen parallelism
[profile.ci]
inherits = "dev"
opt-level = 0
debug = 0
strip = "debuginfo"
incremental = false
codegen-units = 256
```

---

## 8. Sources

- Nextest partitioning docs: https://nexte.st/docs/ci-features/partitioning/
- Nextest configuration reference: https://nexte.st/docs/configuration/reference/
- Nextest test groups: https://nexte.st/docs/configuration/test-groups/
- Nextest build reuse + partition example: https://github.com/nextest-rs/reuse-build-partition-example
- sccache GitHub Action: https://github.com/mozilla-actions/sccache-action
- Swatinem/rust-cache: https://github.com/Swatinem/rust-cache
- Fast Rust Builds with sccache: https://depot.dev/blog/sccache-in-github-actions
- Tips for Faster CI Builds (Corrode): https://corrode.dev/blog/tips-for-faster-ci-builds/
- Tips for Faster Rust Compile Times (Corrode): https://corrode.dev/blog/tips-for-faster-rust-compile-times/
- Cargo profiles reference: https://doc.rust-lang.org/cargo/reference/profiles.html
- Cargo build performance: https://doc.rust-lang.org/cargo/guide/build-performance.html
- Testcontainers for Rust: https://rust.testcontainers.org/
- Testcontainers reusable-containers: https://docs.rs/testcontainers/latest/testcontainers/
- GitHub Actions service containers: https://docs.github.com/en/actions/tutorials/use-containerized-services/create-postgresql-service-containers
- GitHub Actions larger runners: https://docs.github.com/en/actions/using-github-hosted-runners/using-larger-runners
- Testcontainers best practices (Docker): https://www.docker.com/blog/testcontainers-best-practices/
