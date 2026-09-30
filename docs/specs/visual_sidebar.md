# Feature Spec: Visual Sidebar

Endpoint: `GET /fragment/visual-sidebar`

## Scenarios

#### Scenario 32.1: Each Character portrait shows the Character's name

```gherkin
Given a Game whose current Room contains a Character with a name
When the client GET /fragment/visual-sidebar
Then the response body contains one portrait per Character in the Room
And each portrait renders that Character's name as visible label text
```

#### Scenario 32.2: A Character name with HTML-significant characters is escaped

```gherkin
Given a Game whose current Room contains a Character named "Ben & Jerry <Script>"
When the client GET /fragment/visual-sidebar
Then the response body renders the name as escaped text, not markup
And the response body contains no raw "<Script>" tag
```
