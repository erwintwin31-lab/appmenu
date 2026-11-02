# Local browser AI preview

Branch: feat/local-ai-demo. Entry: local-ai.html, linked from the existing demo header. No desktop backend changes. Existing simulated overview remains unchanged; this is an isolated prototype, not the final integrated overview.

Fixed model: Qwen2.5-0.5B-Instruct-q4f16_1-MLC. Runtime: @mlc-ai/web-llm 0.2.84, dynamically imported in a dedicated module worker after consent. Requires HTTPS/localhost, WebGPU and shader-f16. No cloud inference fallback. No analytics added by the preview. Third-party runtime/model downloads expose ordinary request metadata to their hosts. Network and dependency supply-chain review remains required before release.

Five attempted questions per page lifetime, including follow-ups. Refresh resets the limit; not an abuse-prevention boundary. Prompt limit 1200 characters, max output 192 tokens, context 2048 tokens. Loading timeout 10 minutes, generation timeout 2 minutes, idle timeout 3 minutes. Stop, limit, timeout or error terminates the dedicated worker. Browser controls actual memory reclamation; no instantaneous RAM release guarantee. Cached downloads remain and can be removed using browser site-data controls, which may also erase preferences. Estimates in consent are provisional; exact transfer size and peak memory have not been measured.

English preview UI only. Whole-app English default, translated language files/disclaimers, original overview replacement and chat integration remain outstanding. This preview does not fetch live sources or verify generated claims. Do not present its answers as verified or as installation approval.

## Required acceptance before leaving draft

- Browser network trace: opening page and toggling consent must not fetch runtime/model; only Start after consent may do so. Prompts must not appear in outgoing requests.
- Verify runtime CDN import, model availability, license notices, actual transfer size and inference in supported Chrome/Edge versions and target OSes; test unsupported WebGPU and missing shader-f16.
- Verify cancellation during load and generation, timeout handling, repeated clicks, stale worker messages, navigation cleanup and memory behavior.
- Send five questions, including follow-ups; sixth must be disabled and worker stopped. Refresh may reset by design.
- Verify XSS strings render literally, keyboard/screen-reader behavior and narrow-screen header layout.
- Verify no unintended change to existing catalog, simulated installations or desktop builds.
- Evaluate answer accuracy, language behavior, context-overflow behavior and performance on low-memory devices. Long multi-turn input may exceed the token budget; errors must stop cleanly.

No browser inference, download-size measurement, automated tests or full E2E acceptance was performed while preparing these changes. CI and real-device testing are required. This is not a promise of universal device support.
