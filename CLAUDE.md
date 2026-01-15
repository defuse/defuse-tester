# Claude Code Instructions for defuse-tester

## Test Policy

**Never make tests more lenient without asking first.** If a test fails, ask whether to:
1. Fix the code being tested
2. Adjust the test (with explicit approval)

Tests catching differences between implementations is the whole point of this test suite.

## When Tests Unexpectedly Fail

If the user says tests should pass on a target (e.g., "these should all pass on defuse.ca") and they don't:
1. **STOP and report back** - don't silently fix/change the tests
2. List which tests failed and why
3. Ask the user how to proceed (ignore the test, fix test expectations, etc.)

This applies even if the fix seems obvious. The test plan may be wrong, or there may be a reason to diverge from the reference implementation.
