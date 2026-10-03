//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset templates

use askama::Template;

use crate::adapters::driving::http::builders::presets::preset_card_view;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::domain::model::prompt_preset::PromptPreset;

#[derive(Template)]
#[template(
    source = r##"
<div class="prompt-presets-panel">
    <div class="preset-section">
        <h2>System Prompts</h2>
        <p class="preset-section-desc">These prompts are sent as the system message to the narration LLM.</p>
        {% for card in self.system_cards() %}
        {{ card }}
        {% endfor %}

        <details class="preset-add">
            <summary class="btn-cyan preset-add-toggle">Add System Prompt Preset</summary>
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
        </details>
    </div>

    <div class="preset-section">
        <h2>Quantifier Prompts</h2>
        <p class="preset-section-desc">These prompts guide the quantifier LLM that determines NPC presence and player movement.</p>
        {% for card in self.quantifier_cards() %}
        {{ card }}
        {% endfor %}

        <details class="preset-add">
            <summary class="btn-cyan preset-add-toggle">Add Quantifier Prompt Preset</summary>
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
        </details>
    </div>

    <div class="preset-section">
        <h2>Impersonate Prompts</h2>
        <p class="preset-section-desc">These prompts replace the narrator voice when the AI writes as the player's persona via /impersonate. Use the user and persona (persona_description, persona_personality, persona_background) macros to inject persona data.</p>
        {% for card in self.impersonate_cards() %}
        {{ card }}
        {% endfor %}

        <details class="preset-add">
            <summary class="btn-cyan preset-add-toggle">Add Impersonate Prompt Preset</summary>
            <form hx-post="/prompt-presets" hx-target=".prompt-presets-panel" hx-swap="outerHTML">
                <input type="hidden" name="preset_type" value="impersonate" />
                <div class="form-group">
                    <label for="impersonate-preset-name">Name</label>
                    <input type="text" id="impersonate-preset-name" name="name" placeholder="My Custom Impersonate Prompt" required />
                </div>
                <div class="form-group">
                    <label for="impersonate-preset-role">Role</label>
                    <textarea id="impersonate-preset-role" name="role" rows="4" placeholder="Enter role description..."></textarea>
                </div>
                <div class="form-group">
                    <label for="impersonate-preset-instructions">Instructions</label>
                    <textarea id="impersonate-preset-instructions" name="instructions" rows="8" placeholder="Enter instructions..."></textarea>
                </div>
                <div class="form-group">
                    <label for="impersonate-preset-style">Writing Style</label>
                    <textarea id="impersonate-preset-style" name="writing_style" rows="4" placeholder="Enter writing style..."></textarea>
                </div>
                <div class="form-group">
                    <label for="impersonate-preset-output">Output Format</label>
                    <textarea id="impersonate-preset-output" name="output_format" rows="6" placeholder="Enter output format..."></textarea>
                </div>
                <div class="form-actions">
                    <button type="submit" class="btn-primary">Add Preset</button>
                </div>
            </form>
        </details>
    </div>
</div>
"##,
    ext = "html"
)]
pub struct PromptPresetsTemplate {
    pub system_presets: Vec<PromptPreset>,
    pub quantifier_presets: Vec<PromptPreset>,
    pub impersonate_presets: Vec<PromptPreset>,
    pub active_system: ModeActiveIds,
    pub active_quantifier: ModeActiveIds,
    pub active_impersonate: ModeActiveIds,
}

impl PromptPresetsTemplate {
    fn cards(presets: &[PromptPreset], active: &ModeActiveIds) -> Vec<SafeHtml> {
        presets
            .iter()
            .map(|preset| preset_card_view(preset, active))
            .collect()
    }

    /// Rendered cards for the System section.
    pub fn system_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.system_presets, &self.active_system)
    }

    /// Rendered cards for the Quantifier section.
    pub fn quantifier_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.quantifier_presets, &self.active_quantifier)
    }

    /// Rendered cards for the Impersonate section.
    pub fn impersonate_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.impersonate_presets, &self.active_impersonate)
    }
}

/// The active preset id per narrator mode, for one preset type.
#[derive(Debug, Clone)]
pub struct ModeActiveIds {
    pub novel: String,
    pub interactive_fiction: String,
}
