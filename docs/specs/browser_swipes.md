# Feature Spec: Browser Swipes

Endpoint: browser DOM.

## Scenarios

#### Scenario 37.1: Switching to another swipe leaves the dashboard ready

```gherkin
Given the dock shows a generated option set
And the last Message has a second Swipe
When the player switches to the previous Swipe
Then the story log shows the restored Swipe
And #status-display reads "Ready"
And #submit-btn is enabled
And the dock shows no option set
And the restore is announced to the player
```
