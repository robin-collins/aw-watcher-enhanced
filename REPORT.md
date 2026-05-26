# Report: LLM Provider + Qdrant Config Enhancement Assessment

## Scope
This report captures the prior assessment for adding configurable LLM providers (including LM Studio on a remote host) and configurable Qdrant connection settings.

## Key Findings

1. **Config format mismatch (critical):**
   - Documentation references `config.yaml`.
   - Runtime code in `rust-watcher/src/config.rs` loads and writes **`config.toml`** using `toml` serialization/deserialization.
   - This must be resolved before adding new user-facing settings to avoid user confusion and misconfiguration.

2. **LLM implementation is currently Ollama-specific:**
   - `rust-watcher/src/llm.rs` has a fixed `base_url` (`http://localhost:11434`) and an Ollama-specific `/api/generate` request/response contract.
   - There is no provider abstraction yet for OpenAI-compatible endpoints.

3. **No Qdrant runtime config model exists:**
   - Qdrant appears in docs as optional setup.
   - There is no `qdrant` section in runtime config structs and no host/port/auth wiring through config.

## Complexity Estimate
Overall complexity: **Medium** (roughly 1–3 days).

- **Low effort:** add new config structs/fields with defaults and parsing tests.
- **Medium effort:** provider-aware LLM client implementation and compatibility behaviors.
- **Medium effort:** documentation and migration consistency (TOML vs YAML).

## Proposed Task List

1. **Decide canonical config format and enforce consistency**
   - Choose TOML (current runtime) or migrate runtime to YAML.
   - Update docs/install guides accordingly.

2. **Extend LLM config schema**
   - Add `provider` (e.g., `ollama` / `openai_compatible`) and `base_url`.
   - Optionally add API key settings and request timeout controls.

3. **Refactor LLM client for provider routing**
   - Keep Ollama path for backward compatibility.
   - Add OpenAI-compatible request path for LM Studio REST API usage.

4. **Add first-class Qdrant config section**
   - Add `qdrant.enabled`, `host`, `port`, `https`, optional auth/API key, and optional collection name.

5. **Update docs and examples**
   - Include LM Studio remote host example (`http://100.122.201.116:1234`).
   - Add Qdrant config examples.
   - Document effective config filename/format unambiguously.

6. **Add tests and validation**
   - Config default/serialization/partial override tests for new fields.
   - LLM provider routing tests (unit-level).

## Risk Notes

- If format mismatch is not fixed first, support burden and user confusion will remain high.
- OpenAI-compatible endpoints vary subtly by implementation; LM Studio path should be validated against local reference docs in `reference/lmstudio-rest-api/`.
