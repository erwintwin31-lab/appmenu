# Demo AI mode visibility correction

The main search now displays an explicit simulated Preview AI label and a Real local AI entry. It reuses the existing demo/demo.js prepared answers; no new fake generator or model download is used for preview. Local rule-based commands remain available. The old model-preview select in the glass search toolbar is removed from the rendered demo search to avoid confusion. Desktop files are unchanged.

Real local AI opens existing local-ai.html inside a modal iframe. Its consent and compatibility checks precede the existing worker/model import. Closing removes the iframe and navigates it to about:blank; the child page's existing pagehide handler terminates its worker. Actual cleanup and memory reclamation need browser verification. Dialog is appended outside the main render target so parent redraws do not restart an active trial. Model files may remain cached. No automatic fallback silently pretends to be real inference.

This change exposes both modes in the main demo search; it does not rewrite the native AI backend, translate the full UI, validate model answers, or add live web retrieval. The original standalone local-ai.html remains a real-AI-only test page; choose the main search for mode selection.

Required before release: verify header and search entry, preview with WebGPU disabled, no runtime/model requests before consent using a network trace, real inference, five-question limit, close during load/generation, retry by reopening, keyboard/focus behavior across iframe and modal, narrow screens and existing theme/search behavior. Check Pages deployment independently of merge success. No automated/browser/real-device tests have been run for this patch yet.
