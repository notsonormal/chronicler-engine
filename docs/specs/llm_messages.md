# Feature Spec: LLM Messages

Endpoints:
- `POST /action`
- `GET /fragment/llm-messages`

## Scenarios

### Recording every attempt

#### Scenario 33.1: A failed narration attempt is recorded with its failure text and shown in the LLM Messages panel

```gherkin
Given an active Game whose narrator backend fails every narration request
When the client POST /action with command="look"
And the pipeline returns to idle
Then GET /fragment/llm-messages shows one entry for the narrator that names the backend/model the attempt was made against
And that entry carries the backend's failure text
And that entry carries no Raw JSON section
```

#### Scenario 33.2: A later successful attempt does not overwrite an earlier failed attempt

```gherkin
Given an active Game whose narrator backend fails the first narration request and succeeds the second
When the client POST /action with command="look"
And the pipeline returns to idle
And the client POST /action with command="listen"
And the pipeline returns to idle
Then GET /fragment/llm-messages shows two entries for the narrator
And the first entry carries the backend's failure text
And the second entry carries the second narration
```
