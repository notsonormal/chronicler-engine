# Specification: Semantic Navigation

## Objective
The engine's movement system uses quantifier-driven detection. Player types natural language ("I walk through the front gate") and the LLM quantifier detects movement intent, then validates against semantic exits in map.json.

## Current Approach: Quantifier-Driven Movement

1. **All Input is FreeAction**: The parser passes all user input to the LLM as FreeAction.
2. **Quantifier Detection**: After LLM generates narration, the quantifier analyzes the response for movement intent (entering, in, leaving).
3. **Semantic Exits**: Rooms define semantic exits with triggers and keywords for natural language matching.

### Example Flows
- "I walk through the front gate" -> quantifier detects "entering" + "front gate" trigger
- "I head to the kitchen" -> quantifier detects "entering" + "kitchen" trigger (via keywords)

## Semantic Exit Format
Rooms in map.json define semantic exits:
```json
{
  "semantic_exits": [
    {
      "trigger": "front gate",
      "destination": "entrance_hall",
      "keywords": ["enter", "go through", "pass through"]
    }
  ]
}
```

## Resolution Algorithm
1. Quantifier extracts movement intent from LLM response
2. System matches trigger text against current room's semantic exits
3. Keywords enable flexible matching ("go through the front gate" matches "go through")
4. If no match: creates dynamic pseudo-room for invalid destinations
