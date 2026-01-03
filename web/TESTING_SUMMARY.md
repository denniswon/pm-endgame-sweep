# Testing Strategy Summary

## Quick Reference

### Run Tests Locally

```bash
# Unit & Component Tests
yarn test                 # Watch mode
yarn test:coverage        # With coverage report
yarn test:ui              # Interactive UI

# E2E Tests
yarn test:e2e             # Headless mode
yarn test:e2e:ui          # Interactive UI
yarn test:e2e:debug       # Debug mode
```

### Install Test Dependencies

```bash
yarn install
npx playwright install    # Install browser binaries
```

## Test Framework Decision

**Chosen: Vitest + React Testing Library + Playwright**

### Why Vitest?
1. ✅ **Native ESM** - Works perfectly with Next.js 15 and Turbopack
2. ✅ **Fast** - 10-20x faster than Jest for this use case
3. ✅ **Compatible** - Jest-compatible API, easy migration path
4. ✅ **Built-in** - TypeScript, coverage, watch mode, UI
5. ✅ **Modern** - Designed for modern frameworks

### Why React Testing Library?
1. ✅ **User-centric** - Tests behavior, not implementation
2. ✅ **Best practice** - Industry standard for React testing
3. ✅ **Accessible** - Encourages accessible components
4. ✅ **React 19** - Full support for latest React features

### Why Playwright for E2E?
1. ✅ **Cross-browser** - Chrome, Firefox, Safari, Mobile
2. ✅ **Reliable** - Auto-waiting, retry logic
3. ✅ **Developer-friendly** - Debug UI, trace viewer
4. ✅ **Fast** - Parallel execution, efficient
5. ✅ **Modern** - Built for modern web apps

## Alternative Frameworks Considered

### ❌ Jest
- **Pros**: Most popular, huge ecosystem
- **Cons**: Slower, ESM support issues, requires babel/swc transform
- **Why not**: Vitest is faster and better for Next.js 15

### ❌ Cypress
- **Pros**: Great developer experience, time-travel debugging
- **Cons**: Can only test Chrome-based browsers, slower
- **Why not**: Playwright supports more browsers and is faster

### ❌ Testing Library with Jest
- **Pros**: Traditional, well-documented
- **Cons**: Configuration overhead, slower
- **Why not**: Vitest + RTL gives same benefits with better DX

## Test Coverage Goals

```
Overall:     80%+
Utilities:   95%+ (pure functions, critical)
Components:  75%+ (UI components)
API Client:  90%+ (integration points)
E2E:         Critical paths only
```

## Test Distribution (Pyramid)

```
           /\
          /  \     5% E2E
         /____\    (Critical user journeys)
        /      \
       /        \  25% Component/Integration
      /__________\ (UI components, API)
     /            \
    /              \ 70% Unit Tests
   /________________\ (Utilities, pure functions)
```

## What Gets Tested

### ✅ MUST Test
- Pure utility functions (formatPercent, formatDuration, etc.)
- API client functions (network boundaries)
- Component rendering with different props
- User interactions (clicks, form inputs)
- Error states and edge cases
- Critical user flows (E2E)

### ⚠️ OPTIONAL (Lower Priority)
- Type definitions (TypeScript catches these)
- Third-party library wrappers
- Simple pass-through components
- Styling and CSS classes

### ❌ DON'T Test
- External libraries (trust them)
- Next.js framework code
- Simple constants/configs
- Auto-generated code

## Test File Structure

```
web/
├── lib/
│   ├── utils.ts
│   └── utils.test.ts              ✅ Unit tests
├── components/
│   └── ui/
│       ├── badge.tsx
│       └── badge.test.tsx         ✅ Component tests
├── tests/
│   └── setup.ts                   ✅ MSW + test config
├── e2e/
│   ├── opportunities.spec.ts      ✅ E2E tests
│   └── market-details.spec.ts     ✅ E2E tests
├── vitest.config.ts               ⚙️ Vitest config
└── playwright.config.ts           ⚙️ Playwright config
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: 'yarn'

      - name: Install dependencies
        run: |
          cd web
          yarn install --frozen-lockfile

      - name: Run unit tests
        run: |
          cd web
          yarn test:coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./web/coverage/coverage-final.json

      - name: Install Playwright browsers
        run: |
          cd web
          npx playwright install --with-deps

      - name: Run E2E tests
        run: |
          cd web
          yarn test:e2e

      - name: Upload test results
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: web/playwright-report/
```

## Current Test Coverage

### Files with Tests ✅
- `lib/utils.ts` - 35 unit tests
- `components/ui/badge.tsx` - 8 component tests
- `lib/api.ts` - 20 API integration tests
- E2E: Opportunities page - 10 tests
- E2E: Market details page - 8 tests

**Total: 81 tests**

### Files Without Tests (TODO)
- `components/ui/card.tsx`
- `components/ui/table.tsx`
- `components/ui/skeleton.tsx`
- `components/features/opportunities-table.tsx`
- `components/features/market-card.tsx`
- `app/page.tsx`
- `app/market/[id]/page.tsx`

## Next Steps

1. **Install dependencies**: `yarn install && npx playwright install`
2. **Run existing tests**: `yarn test` (should pass)
3. **Add more component tests**: Test OpportunitiesTable, MarketCard
4. **Run E2E tests**: `yarn test:e2e` (requires backend running)
5. **Set up CI/CD**: Add GitHub Actions workflow
6. **Monitor coverage**: Aim for 80%+ overall

## Performance Benchmarks

### Vitest (Unit + Component)
- ~1000 tests: < 5 seconds
- Watch mode: < 100ms for single file change
- Coverage report: + 2-3 seconds

### Playwright (E2E)
- 10 tests across 3 browsers: ~30 seconds
- Parallel execution: ~10 seconds
- Single browser: ~10 seconds

## Best Practices Checklist

- ✅ Test user behavior, not implementation
- ✅ Use semantic queries (getByRole > getByTestId)
- ✅ Keep tests simple and readable
- ✅ One assertion per test (when possible)
- ✅ Descriptive test names
- ✅ Arrange-Act-Assert pattern
- ✅ Test isolation (no shared state)
- ✅ Mock external dependencies
- ✅ Fast feedback (< 1s for unit tests)
- ✅ Reliable (no flaky tests)

## Resources

- [TESTING.md](./TESTING.md) - Complete testing guide
- [Vitest Docs](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Playwright Docs](https://playwright.dev/)
- [MSW Docs](https://mswjs.io/)

## FAQ

**Q: Why not Jest?**
A: Vitest is faster, has better ESM support, and works better with Next.js 15.

**Q: Do I need to run E2E tests locally?**
A: Optional. Run them before pushing major changes. CI will run them automatically.

**Q: How do I debug a failing test?**
A: Use `yarn test:ui` for unit tests, `yarn test:e2e:debug` for E2E tests.

**Q: What if coverage drops below 80%?**
A: CI will fail. Add tests for uncovered code or adjust thresholds if justified.

**Q: Should I test styling?**
A: No, test behavior. Use visual regression tools for styling (not included here).

**Q: How do I test SWR hooks?**
A: Wrap components in SWRConfig with test cache, as shown in examples.

**Q: Should I mock Next.js router?**
A: Only when necessary. Use Playwright for true navigation testing.
