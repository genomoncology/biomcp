# Test Environment Configuration

This document explains how test environment variables are managed to ensure deterministic, reproducible tests.

## The Problem

Cloudflare's `@cloudflare/vitest-pool-workers` loads environment variables from multiple sources:

1. `wrangler.jsonc` vars (base configuration)
2. `.dev.vars` (local development overrides)
3. `vitest.config.mts` miniflare bindings (test-specific overrides)

By default, `.dev.vars` is loaded, which means **local development settings can break tests**. For example, if a developer has `IDENTITY_PROVIDER=oauth` in their `.dev.vars` but tests expect `stytch`, tests will fail.

## The Solution

We use Cloudflare's environment-specific `.dev.vars` feature:

```text
.dev.vars       → Loaded during `pnpm dev` (gitignored)
.dev.vars.test  → Loaded during `pnpm test` (committed)
```

The `vitest.config.mts` specifies `environment: 'test'`:

```typescript
export default defineWorkersConfig({
  test: {
    poolOptions: {
      workers: {
        wrangler: {
          configPath: './wrangler.jsonc',
          environment: 'test', // Load .dev.vars.test instead of .dev.vars
        },
      },
    },
  },
});
```

When `environment: 'test'` is set, wrangler loads `.dev.vars.test` **instead of** `.dev.vars` (not merged).

### How `.dev.vars.test` Values Reach Test Code

1. **Wrangler loads `.dev.vars.test`** when `environment: 'test'` is set in vitest.config.mts
2. **These values become bindings** in the miniflare environment
3. **`cloudflare:test` exports an `env` object** with these bindings (typed via `test/env.d.ts`)
4. **`createTestEnv()` spreads this `env`** with custom overrides

```typescript
// In test/helpers/env.ts
import { env } from 'cloudflare:test'; // Contains .dev.vars.test values

export function createTestEnv(overrides: Partial<RawEnv> = {}): RawEnv {
  return { ...env, ...overrides };
}
```

## File Purposes

| File                | Purpose                          | Git Status |
| ------------------- | -------------------------------- | ---------- |
| `.dev.vars`         | Local development secrets        | Ignored    |
| `.dev.vars.example` | Template showing required vars   | Committed  |
| `.dev.vars.test`    | Test environment (deterministic) | Committed  |

## Adding New Environment Variables

When adding a new environment variable:

1. **Add to `wrangler.jsonc`** with a placeholder or default value
2. **Add to `.dev.vars.example`** with documentation
3. **Add to `.dev.vars.test`** with a test-safe value
4. **Update `test/env.d.ts`** if the variable should be typed in tests

### Example: Adding a New Variable

```jsonc
// wrangler.jsonc
{
  "vars": {
    "NEW_FEATURE_FLAG": "false",
  },
}
```

```bash
# .dev.vars.example
NEW_FEATURE_FLAG=true  # Enable new feature (default: false)
```

```bash
# .dev.vars.test
NEW_FEATURE_FLAG=false  # Use stable default for tests
```

```typescript
// test/env.d.ts
declare module 'cloudflare:test' {
  // Extends base Env with test-specific bindings from .dev.vars.test
  interface ProvidedEnv extends Env {
    // ... existing vars
    NEW_FEATURE_FLAG?: string;
  }
}
```

## Test Helpers

Use `createTestEnv()` and `parseTestEnv()` from `test/helpers/env.ts` to create test environments with custom overrides:

```typescript
import { createTestEnv, parseTestEnv } from './helpers/env';

// Override specific values for a test
const env = createTestEnv({
  IDENTITY_PROVIDER: 'oauth',
  OAUTH_CLIENT_ID: 'test-client',
});

// Get parsed env with overrides
const parsed = parseTestEnv({
  REMOTE_MCP_AUTH_TOKEN: 'custom-token-for-this-test!!!',
});
```

These helpers merge:

1. Real bindings from `cloudflare:test` (populated from `.dev.vars.test`)
2. Your custom overrides

## Troubleshooting

### Tests fail locally but pass in CI

Your `.dev.vars` likely has values that differ from `.dev.vars.test`. The fix is already in place—tests use `.dev.vars.test`. If you still see issues:

1. Verify `vitest.config.mts` has `environment: 'test'`
2. Check that `.dev.vars.test` exists and has the expected values
3. Look for wrangler output: `Using vars defined in .dev.vars.test`

### Tests need a different identity provider

Some tests may need OAuth instead of Stytch. Use `createTestEnv()`:

```typescript
it('tests OAuth-specific behavior', async () => {
  const env = createTestEnv({
    IDENTITY_PROVIDER: 'oauth',
    OAUTH_CLIENT_ID: 'test-client',
    OAUTH_CLIENT_SECRET: 'test-secret',
    OAUTH_AUTHORIZATION_URL: 'https://oauth.example.com/authorize',
    OAUTH_TOKEN_URL: 'https://oauth.example.com/token',
  });

  // Use env in your test...
});
```

### New binding not available in tests

1. Add it to `.dev.vars.test`
2. Add the type to `test/env.d.ts`
3. If using `createTestEnv()`, the binding will be available via the `env` import

## Why Not Use `miniflare.bindings`?

We previously used `miniflare.bindings` in `vitest.config.mts` to override specific values. The `.dev.vars.test` approach is better because:

1. **Complete replacement**: `.dev.vars.test` replaces `.dev.vars` entirely, no merge conflicts
2. **Self-documenting**: All test env vars in one visible file
3. **Follows Cloudflare's pattern**: Uses the intended environment feature
4. **Easier to maintain**: Add new vars in one place, not two

## Related Documentation

- [Environment Configuration](./environment-configuration.md) - Adding and validating env vars
- [Workers Caching](./workers-caching.md) - Module vs instance-level state in tests
