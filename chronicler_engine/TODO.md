# QUICK TODO List

# Added: 2026-02-21

- [ ] Review where the open api key variable is called/set. In principle, we should only need to pick it up in the openrouter client but it's being picked up outside of it
- [ ] Test that the feature `reactive-movement.md` is working correctly (It doesn't seem to...)
    - [ ] Review the structure the continuation system prompt. It should be in XML and it should be documented like the others.
    - [ ] Check how the trigger system works. I know if seems to always require the room to the correct set, which isn't what I want since that it just a specific trigger.
    - [ ] Times met should always be set, even outside of the trigger. Although it's going to have to be based on when some meets up for the first time, or leaves and comes back (e.g. not just Carla following you around)
- [ ] Going to need to consider multiple system prompts, or rewrite the existing ones with "Do not speak for the user" type stuff.