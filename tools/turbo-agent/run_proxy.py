"""Launch the turbo-agent proxy with the Synthetic-compatible verifier patch.

Why the patch exists: llm_verifier's default OpenAI path (call_openai) always
runs a vLLM-only "prefill" call (continue_final_message + structured_outputs).
Synthetic accepts those fields but silently ignores them, so the prefilled
score distribution is garbage and every tournament comparison scores 0.5,
which turns selection into a coin flip.

Setting _llm_verifier_deepseek on the client routes scoring through
call_deepseek instead, which reads the distribution from the model's own
sampled <score_X> tags (GLM-5.3-Flash emits these itself) and never prefills.
Despite the name, the flag means "self-tagging backend", not "DeepSeek only".
"""
import turbo_agent.utils.verifier_client as vc

_orig = vc.build_verifier_client


def patched(cfg):
    client = _orig(cfg)
    if client is not None and hasattr(client, "chat"):
        client._llm_verifier_deepseek = True
        print("[patch] verifier client: skip prefill, read model's own tags")
    return client


vc.build_verifier_client = patched

# Consumers bind the name at import time; rebind it where it is used.
import turbo_agent.verifier.verifier as _verifier_module  # noqa: E402

if hasattr(_verifier_module, "build_verifier_client"):
    _verifier_module.build_verifier_client = patched

try:
    import turbo_agent.progress_monitor.monitor as _monitor_module  # noqa: E402

    if hasattr(_monitor_module, "build_verifier_client"):
        _monitor_module.build_verifier_client = patched
except ImportError:
    pass

import llm_verifier as _llm_verifier  # noqa: E402

_orig_select = _llm_verifier.select


def _select_capped(*args, **kwargs):
    # Match Synthetic's ~2-concurrent-request limit; anything higher draws 429s.
    kwargs.setdefault("max_workers", 2)
    return _orig_select(*args, **kwargs)


_llm_verifier.select = _select_capped

from turbo_agent.cli import main  # noqa: E402

main()
