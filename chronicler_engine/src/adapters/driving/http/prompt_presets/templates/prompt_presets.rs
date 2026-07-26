//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset templates

use askama::Template;

use crate::domain::model::prompt_preset::PromptPreset;

#[derive(Template)]
#[template(
    source = r#"
<div class="prompt-presets-panel">
    <div class="preset-section">
        <h2>System Prompts</h2>
        <p class="preset-section-desc">These prompts are sent as the system message to the narration LLM.</p>
        {% for preset in system_presets %}
        <div class="preset-card{% if preset.is_default %} default{% endif %}{% if preset.id == active_system_id %} active{% endif %}">
            <div class="card-header">
                <span class="card-title">{{ preset.name }}</span>
                <div class="card-badges">
                    {% if preset.is_default %}<span class="badge">Default</span>{% endif %}
                    {% if preset.id == active_system_id %}<span class="badge primary">Active</span>{% endif %}
                </div>
            </div>
            <div class="card-details preset-preview">{{ preset.preview_text() | escape }}</div>
            <div class="card-actions">
                {% if preset.id != active_system_id %}
                <button hx-post="/prompt-presets/{{ preset.id }}/activate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active</button>
                {% endif %}
                {% if preset.is_default %}
                <button hx-get="/fragment/prompt-presets/{{ preset.id }}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>
                {% else %}
                <button hx-get="/fragment/prompt-presets/{{ preset.id }}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button>
                <button hx-post="/prompt-presets/{{ preset.id }}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>
                {% endif %}
                <button hx-post="/prompt-presets/{{ preset.id }}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button>
            </div>
        </div>
        {% endfor %}

        <h3>Add System Prompt Preset</h3>
        <form hx-post="/prompt-presets" hx-target=".prompt-presets-panel" hx-swap="outerHTML">
            <input type="hidden" name="preset_type" value="system" />
            <div class="form-group">
                <label for="system-preset-name">Name</label>
                <input type="text" id="system-preset-name" name="name" placeholder="My Custom System Prompt" required />
            </div>
            <div class="form-group">
                <label for="system-preset-role">Role</label>
                <textarea id="system-preset-role" name="role" rows="4" placeholder="Enter role description..."></textarea>
            </div>
            <div class="form-group">
                <label for="system-preset-instructions">Instructions</label>
                <textarea id="system-preset-instructions" name="instructions" rows="8" placeholder="Enter instructions..."></textarea>
            </div>
            <div class="form-group">
                <label for="system-preset-style">Writing Style</label>
                <textarea id="system-preset-style" name="writing_style" rows="4" placeholder="Enter writing style..."></textarea>
            </div>
            <div class="form-group">
                <label for="system-preset-output">Output Format</label>
                <textarea id="system-preset-output" name="output_format" rows="6" placeholder="Enter output format..."></textarea>
            </div>
            <div class="form-actions">
                <button type="submit" class="btn-primary">Add Preset</button>
            </div>
        </form>
    </div>

    <div class="preset-section">
        <h2>Quantifier Prompts</h2>
        <p class="preset-section-desc">These prompts guide the quantifier LLM that determines NPC presence and player movement.</p>
        {% for preset in quantifier_presets %}
        <div class="preset-card{% if preset.is_default %} default{% endif %}{% if preset.id == active_quantifier_id %} active{% endif %}">
            <div class="card-header">
                <span class="card-title">{{ preset.name }}</span>
                <div class="card-badges">
                    {% if preset.is_default %}<span class="badge">Default</span>{% endif %}
                    {% if preset.id == active_quantifier_id %}<span class="badge primary">Active</span>{% endif %}
                </div>
            </div>
            <div class="card-details preset-preview">{{ preset.preview_text() | escape }}</div>
            <div class="card-actions">
                {% if preset.id != active_quantifier_id %}
                <button hx-post="/prompt-presets/{{ preset.id }}/activate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active</button>
                {% endif %}
                {% if preset.is_default %}
                <button hx-get="/fragment/prompt-presets/{{ preset.id }}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>
                {% else %}
                <button hx-get="/fragment/prompt-presets/{{ preset.id }}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button>
                <button hx-post="/prompt-presets/{{ preset.id }}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>
                {% endif %}
                <button hx-post="/prompt-presets/{{ preset.id }}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button>
            </div>
        </div>
        {% endfor %}

        <h3>Add Quantifier Prompt Preset</h3>
        <form hx-post="/prompt-presets" hx-target=".prompt-presets-panel" hx-swap="outerHTML">
            <input type="hidden" name="preset_type" value="quantifier" />
            <div class="form-group">
                <label for="quantifier-preset-name">Name</label>
                <input type="text" id="quantifier-preset-name" name="name" placeholder="My Custom Quantifier Prompt" required />
            </div>
            <div class="form-group">
                <label for="quantifier-preset-role">Role</label>
                <textarea id="quantifier-preset-role" name="role" rows="4" placeholder="Enter role description..."></textarea>
            </div>
            <div class="form-group">
                <label for="quantifier-preset-instructions">Instructions</label>
                <textarea id="quantifier-preset-instructions" name="instructions" rows="8" placeholder="Enter instructions..."></textarea>
            </div>
            <div class="form-group">
                <label for="quantifier-preset-output">Output Format</label>
                <textarea id="quantifier-preset-output" name="output_format" rows="6" placeholder="Enter output format..."></textarea>
            </div>
            <div class="form-actions">
                <button type="submit" class="btn-primary">Add Preset</button>
            </div>
        </form>
    </div>
</div>
"#,
    ext = "html"
)]
pub struct PromptPresetsTemplate {
    pub system_presets: Vec<PromptPreset>,
    pub quantifier_presets: Vec<PromptPreset>,
    pub active_system_id: String,
    pub active_quantifier_id: String,
}
