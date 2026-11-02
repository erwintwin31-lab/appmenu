# Local browser AI preview

Branch: feat/local-ai-demo. Entry: local-ai.html, linked from the existing demo header. No desktop backend changes. Existing simulated overview remains unchanged; this is an isolated prototype, not the final integrated overview.

## Modes

- Preview AI (no download): always available, no model download, no WebGPU requirement and no on-device inference. Uses prepared demo answers and is permanently labeled “Preview · simulated answers”. Intended for restricted work devices and interface inspection. No five-question limit.
- Real local AI: requires explicit consent, then downloads and runs Qwen2.5-0.5B-Instruct-q4f16_1-MLC through WebLLM 0.2.84 in a dedicated worker. Requires HTTPS/localhost, WebGPU and shader-f16. Five attempted questions per page lifetime, including follow-ups. Refresh resets the limit; not an abuse-prevention boundary.

No cloud inference fallback. No analytics added by the preview. Third-party runtime/model downloads expose ordinary request metadata to their hosts. Network and dependency supply-chain review remains required before release.

Prompt limit 1200 characters, max output 192 tokens, context 2048 tokens. Loading timeout 10 minutes, generation timeout 2 minutes, idle timeout 3 minutes. Stop, limit, timeout or error terminates the dedicated worker. Browser controls actual memory reclamation; no instantaneous RAM release guarantee. Cached downloads remain and can be removed using browser site-data controls, which may also erase preferences. Estimates in consent are provisional; exact transfer size and peak memory have not been measured.

English preview UI only. Whole-app English default, translated language files/disclaimers, original overview replacement and chat integration remain outstanding. Neither mode fetches live sources or verifies generated claims. Preview answers must never be presented as model-generated or as installation approval.

## Required acceptance before leaving draft

- Opening the page and choosing Preview AI must not request the WebLLM runtime or model. Verify in a browser network trace.
- Preview mode must display the simulated-answer badge and note, work without WebGPU, and impose no question limit.
- Real mode must not load runtime/model before consent and Start. Verify prompts are not sent to remote inference and that worker termination occurs on stop, timeout, error or the fifth attempted question.
- Verify runtime CDN import, model availability, license notices, actual transfer size and inference in supported Chrome/Edge versions and target OSes; test unsupported WebGPU and missing shader-f16.
- Verify cancellation during load and generation, timeout handling, repeated clicks, stale worker messages, navigation cleanup and memory behavior.
- Verify XSS strings render literally, keyboard/screen-reader behavior and narrow-screen layout.
- Verify no unintended change to existing catalog, simulated installations or desktop builds.
- Evaluate real-model answer accuracy, language behavior, context-overflow behavior and performance on low-memory devices. Long multi-turn input may exceed the token budget; errors must stop cleanly.

No browser inference, download-size measurement, automated tests or full E2E acceptance was performed while preparing these changes. CI and real-device testing are required. This is not a promise of universal device support.
