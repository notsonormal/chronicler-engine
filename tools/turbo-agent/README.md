# Turbo-agent proxy (verified responses for pi)

> **Status: retired 2026-09-05 (soft).** Experiment concluded negatively:
> interactive latency is a no-go (turns exceed pi's 300s idle timeout on
> large sessions), "deep think" use is redundant with a stronger model in
> one call (GLM 5.2 / Kimi K3), and on long contexts the ~9x context
> re-read makes it slow AND expensive. Kept as the written record of the
> findings + revived with `turbo-proxy.sh start` if a batch/agentic
> workload (where per-step error correction is the actual premise) ever
> appears. Not committed to git.


Runs the [TurboAgent](https://github.com/llm-as-a-verifier/TurboAgent) LLM API
proxy against the Synthetic provider. The proxy sits between pi and Synthetic:
every request fans out into N parallel candidates, a probabilistic pivot
tournament scores them with token-level logprobs, and the best response is
returned to pi as one message.

```
pi request -> [proxy] -> N parallel candidates (GLM-5.3-Flash on Synthetic)
        -> pivot tournament (logprob verifier, same model)
        -> best response -> pi
```

This is an experiment, not part of the engine build. `build.py` does not
touch this directory.

## Why the patch

`run_proxy.py` sets `_llm_verifier_deepseek` on the verifier client. Without
it, llm_verifier's default OpenAI path runs a vLLM-only "prefill" call
(`continue_final_message` + `structured_outputs`). Synthetic accepts those
fields but ignores them, so every score degrades to 0.5 and selection becomes
random. The flag routes scoring through the "read the model's own sampled
score tags" path, which GLM-5.3-Flash supports (verified 2026-08-31: real
letter distributions, non-degenerate tournament).

## Usage

```bash
tools/turbo-agent/turbo-proxy.sh start     # also creates .venv on first run
tools/turbo-agent/turbo-proxy.sh status
tools/turbo-agent/turbo-proxy.sh stop
```

The script reads the Synthetic key from `~/.pi/agent/auth.json` at launch.
The key is never written to disk here (no `.env` files).

Defaults: port 8899, 3 candidates, 1 verification per pair. See
`turbo-agent.yaml`. 5 candidates is not viable: Synthetic answers 429
`Too many concurrent requests` above ~5 requests in flight (observed 2026-09-05:
2 of 5 candidates rejected), so best-of-5 degraded to best-of-3 plus wasted
retries. `run_proxy.py` also caps the tournament pool to 2 workers —
llm_verifier defaults to 50, which fires every comparison at once.

## Pointing pi at it

`~/.pi/agent/models.json` (global — pi has no project-scoped provider config):

```json
"turbo": {
  "name": "Turbo (verified)",
  "baseUrl": "http://127.0.0.1:8899/v1",
  "api": "openai-completions",
  "apiKey": "local-proxy",
  "compat": { "supportsDeveloperRole": false },
  "models": [
    {
      "id": "glm-5.3-flash-verified",
      "name": "GLM 5.3 Flash (LLM-as-verifier best-of-5)",
      "reasoning": true,
      "input": ["text"],
      "contextWindow": 524288
    }
  ]
}
```

Two non-obvious constraints:

- `apiKey` must be present and non-empty. pi drops providers without
  configured auth at composition time (provider-composer.js throws, the
  runtime deletes the provider). The proxy ignores auth; any placeholder
  works.
- The model `id` is arbitrary — the proxy never reads the incoming `model`
  field and always uses `backend.models[].name` from its own yaml. Keep the
  id slash-free so pi's `provider/id` addressing stays unambiguous.

Select the `turbo` provider in pi for long-running tasks where multi-minute
turns are acceptable.

## Cost and latency (measured 2026-09-05, Synthetic GLM-5.3-Flash)

- Best-of-3, small prompt: 62s (was 358s for best-of-5; candidates 9s,
  tournament 53s for 6 comparisons).
- Real pi turns re-send the full conversation in every candidate and every
  comparison. At a 74-message / ~33k-token session this was 2-4.5 min per
  step under best-of-5; best-of-3 roughly halves it, and it grows with
  session length.
- Generation and verification share Synthetic's 2-concurrent-request limit.
- Verifier cost is input-heavy (~20 trace re-reads per task); at $0.15/$0.04/
  $0.50 per mtok this is ~$0.4/task for small trajectories. GLM-5.2 as the
  verifier would cost ~5.5x more per call.
