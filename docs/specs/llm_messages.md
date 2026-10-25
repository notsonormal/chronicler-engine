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
And the Game is no longer busy
Then GET /fragment/llm-messages shows one entry for the narrator that names the backend/model the attempt was made against
And that entry carries the backend's failure text
And that entry carries no Raw JSON section
```

#### Scenario 33.2: A later successful attempt does not overwrite an earlier failed attempt

```gherkin
Given an active Game whose narrator backend fails the first narration request and succeeds the second
When the client POST /action with command="look"
And the Game is no longer busy
And the client POST /action with command="listen"
And the Game is no longer busy
Then GET /fragment/llm-messages shows two entries for the narrator
And the first entry carries the backend's failure text
And the second entry carries the second narration
```

#### Scenario 33.3: The LLM Messages fragment renders each message collapsed, with its view state listed in data-keep

```gherkin
Given an active Game with one recorded LLM attempt
When the client GET /fragment/llm-messages
Then the response status is "200 OK"
And the body does not hold an element with the class "llm-messages-panel"
And the attempt's card has the id "llm-msg-{id}"
And the attempt's header reports aria-expanded="false" and lists "aria-expanded" in `data-keep`
And the attempt's body is `hidden` and lists "hidden" in `data-keep`
And each prompt `details` lists "open" in `data-keep`
```
