# Feature Spec: Swipe Switching

Endpoint: `POST /message/:id/swipe/:index`

## Scenarios

### Restoring a swipe

#### Scenario 36.1: Switching to another swipe leaves the game ready

```gherkin
Given a game whose last Message has two Swipes
When the client switches to the other Swipe
And the client reads the generation status
Then the generation status is idle
```

#### Scenario 36.2: Switching to another swipe drops the offered options

```gherkin
Given the dock shows a generated option set
And the last Message has two Swipes
When the client switches to the other Swipe
Then the dock renders no options strip
```
