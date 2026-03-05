# Integration Tests for defuse.ca

A mostly vibe-coded suite of tests for [defuse.ca](https://defuse.ca/).

To run the tests, put the captcha bypass secret into `secrets/captcha-bypass-key.txt`, and then run:

```bash
DEFUSE_URL=http://localhost:3000/ cargo test --no-fail-fast  -- --include-ignored
```