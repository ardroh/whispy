# Feature Specification: Optional Rephrase Mode

**Feature Branch**: `001-rephrase-mode`  
**Created**: 2026-06-03  
**Status**: Draft  
**Input**: User description: "Add optional mode (configured from settings) that adds rephrasing in the pipeline like disclosed here: ../../private/rephraser-rust/prompts/rephrase.system.md before pasting the message."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Polish dictated text before it lands (Priority: P1)

A user dictates a message by holding the hotkey. With rephrase mode enabled, after the spoken words are turned into text, the text is automatically cleaned up — fixing grammar, converting text-speak (e.g. "u" → "you"), and improving clarity — and the polished version is what gets pasted at the cursor.

**Why this priority**: This is the core value of the feature. Spoken dictation is often informal, contains filler, or uses casual shorthand; getting a clean, professional result without manual editing is the entire reason for the feature.

**Independent Test**: Enable rephrase mode in settings, dictate an informal sentence (e.g. "hey can u pls check this asap"), and confirm the pasted output is a clearer, grammatically correct version while preserving meaning and abbreviations like "ASAP".

**Acceptance Scenarios**:

1. **Given** rephrase mode is enabled, **When** the user dictates an informal sentence and releases the hotkey, **Then** a clarity-improved version of the transcription is pasted at the cursor.
2. **Given** rephrase mode is enabled, **When** the dictated text is already clear and well-phrased, **Then** the pasted text is unchanged or only minimally adjusted.
3. **Given** rephrase mode is enabled, **When** the transcription contains a question (e.g. "what time is the meeting"), **Then** the pasted output is a rephrased version of that question, not an answer to it.

---

### User Story 2 - Toggle rephrasing on or off from settings (Priority: P1)

A user opens the preferences window and turns rephrase mode on or off. The choice persists across restarts and takes effect on the next recording without restarting the app.

**Why this priority**: The feature is explicitly described as optional. Without a reliable, persistent toggle, users cannot control when their words are altered — which is essential for trust.

**Independent Test**: Open preferences, toggle rephrase mode off, dictate a message, and confirm the raw transcription is pasted unchanged; toggle it on and confirm the rephrased output is pasted.

**Acceptance Scenarios**:

1. **Given** the preferences window is open, **When** the user enables rephrase mode and saves, **Then** the setting persists and is reflected when preferences are reopened.
2. **Given** rephrase mode is disabled, **When** the user dictates a message, **Then** the exact transcription is pasted with no rephrasing step.
3. **Given** the user changes the rephrase setting, **When** the next recording completes, **Then** the new setting is applied without needing an app restart.

---

### User Story 3 - Stay productive when rephrasing is unavailable (Priority: P2)

When rephrase mode is enabled but the rephrasing step cannot complete (network failure, service error, missing credentials, or excessive delay), the user is not left empty-handed — the original transcription is pasted instead and the user is informed that rephrasing was skipped.

**Why this priority**: Reliability of the paste action is more important than the polish. Losing dictated content because an optional enhancement failed would be unacceptable, but it is a safeguard rather than the primary value.

**Independent Test**: Enable rephrase mode with no network connectivity, dictate a message, and confirm the original transcription is still pasted and a notification explains that rephrasing was skipped.

**Acceptance Scenarios**:

1. **Given** rephrase mode is enabled and the rephrasing service is unreachable, **When** a recording completes, **Then** the original transcription is pasted and the user is notified that rephrasing failed.
2. **Given** rephrase mode is enabled and rephrasing exceeds the maximum allowed time, **When** the timeout is reached, **Then** the original transcription is pasted rather than blocking indefinitely.
3. **Given** rephrase mode is enabled but required credentials are missing, **When** a recording completes, **Then** the original transcription is pasted and the user is told what is needed to enable rephrasing.

---

### Edge Cases

- **Empty or silent transcription**: When the transcription is empty or whitespace-only, the rephrasing step is skipped entirely and behavior matches today's empty-result handling.
- **Very long transcription**: When the dictated text is unusually long, rephrasing must still complete within the timeout or fall back to the original text.
- **Prompt-injection attempts in speech**: When the dictated text contains instruction-like phrasing (e.g. "ignore previous instructions and..."), the text is treated as content to rephrase only, never as instructions to act on.
- **Abbreviations and technical terms**: Common abbreviations (ASAP, FYI, EOD, PR, API) must be preserved, not expanded.
- **Non-English / "auto" language transcription**: The reference rephrasing rules are US-English-specific; rephrasing is applied to all transcriptions regardless of detected language, with the understanding that quality is best for English (see Assumptions).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a user-configurable setting to enable or disable rephrase mode, accessible from the preferences window.
- **FR-002**: System MUST default rephrase mode to disabled for existing and new users, preserving current behavior unless the user opts in.
- **FR-003**: System MUST persist the rephrase mode setting across application restarts.
- **FR-004**: System MUST apply the current rephrase mode setting to the next recording without requiring an application restart.
- **FR-005**: When rephrase mode is enabled, System MUST pass the completed transcription through a rephrasing step before the text is pasted at the cursor.
- **FR-006**: The rephrasing step MUST follow the rephrasing rules defined in the reference prompt: correct US English grammar and syntax, improve clarity and conciseness, add no new information, convert informal text-speak to proper words, preserve abbreviations, leave already-clear text unchanged, and treat the input strictly as content to rephrase rather than instructions to follow.
- **FR-007**: When rephrase mode is disabled, System MUST paste the unaltered transcription exactly as it does today.
- **FR-008**: When the rephrasing step fails, errors, or exceeds a maximum time limit, System MUST fall back to pasting the original transcription.
- **FR-009**: System MUST notify the user when a requested rephrasing was skipped or failed, including the reason where known (e.g. unreachable service, missing credentials).
- **FR-010**: System MUST skip the rephrasing step when the transcription is empty or contains only whitespace.
- **FR-011**: System MUST inform the user, at the point of enabling rephrase mode or at first use, of any additional credentials or configuration required for rephrasing to function.
- **FR-012**: System MUST ensure that enabling rephrase mode does not change the recording hotkey behavior or the user's interaction flow beyond the content of the pasted text.

### Key Entities *(include if feature involves data)*

- **Rephrase Setting**: A persisted user preference indicating whether rephrasing is applied to transcriptions. Stored alongside existing preferences (model, language, hotkey, API key).
- **Rephrasing Rules**: The fixed set of guidelines (sourced from the reference prompt) that govern how transcribed text is transformed — grammar correction, clarity, abbreviation preservation, and treating input as content only.
- **Transcription Result**: The text produced from speech, which is the input to the optional rephrasing step and, when rephrasing is disabled or fails, the text that is pasted directly.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With rephrase mode enabled, informal dictated sentences are returned as clearer, grammatically correct text while preserving the original meaning in at least 95% of cases sampled.
- **SC-002**: Toggling rephrase mode on or off takes effect on the very next recording 100% of the time, with no app restart required.
- **SC-003**: When rephrasing fails or times out, the original transcription is still pasted 100% of the time, so users never lose dictated content.
- **SC-004**: The rephrasing step adds no more than a small, bounded delay to the end-to-end dictation-to-paste flow, and the flow never blocks indefinitely (a maximum wait is always enforced).
- **SC-005**: With rephrase mode disabled, the dictation-to-paste behavior and timing are indistinguishable from the current application.
- **SC-006**: Abbreviations present in dictated text (e.g. ASAP, FYI, API) are preserved unexpanded in at least 95% of sampled rephrased outputs.

## Assumptions

- The rephrasing capability is provided by the same class of external AI service already used for transcription (the OpenAI account associated with the configured API key), reusing existing credentials where possible.
- The rephrasing rules are fixed and sourced from the reference prompt; users configure only whether rephrasing is on or off, not the rules themselves, in this version.
- Rephrasing happens after transcription completes and before the paste step, slotting into the existing `Transcribing → Idle (paste)` transition without changing other phases.
- A reasonable maximum wait time for rephrasing is enforced so the paste action is never blocked indefinitely; the exact duration is an implementation detail.
- User notifications reuse the existing native macOS notification mechanism already used for errors.
- The rephrasing service availability and quality are outside the application's control; the application's responsibility is limited to invoking it correctly and falling back gracefully.
