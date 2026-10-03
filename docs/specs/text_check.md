# Feature Spec: Text Check

Endpoint: `POST /check-text`

## Scenarios

### Log-entry check

#### Scenario 34.1: A log-entry check returns a read-only result panel

```gherkin
Given a game state whose story log holds an entry with a misspelling
And text check mode is Spell
When the client POST /check-text for that entry
Then the response is the read-only result panel rendered into #text-check-result
And the panel names the entry it checked
And the panel offers no send form
And the panel is not the send preview
```

The panel is read-only. It reports suggestions for the entry's text and
leaves the entry unchanged.
