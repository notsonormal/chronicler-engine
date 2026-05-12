# QUICK TODO List

# Added: 2026-05-12

- [ ] The error (the red run over the top) overlaps the [Reset Button]. It also disappears and reappears every time it loads
- [ ] We should be able to retry to first LLM message after the player input but we get the error "Error: Retry failed: no pre-generation snapshot found". We need to rethink how that works.
- [ ] Character should not be disappearing and appearing on the UI as it processes events, it's a bit annoying
- [ ] For characters, we only include their relationship with other characters if they are in the room. It would probably be better to include all their relationships, although that makes things complicated, since we won't have the full character card to establish context for that relationship.
    - Perhaps we also should include full character cards if we think they are relevant to the current scene, even if they aren't present?