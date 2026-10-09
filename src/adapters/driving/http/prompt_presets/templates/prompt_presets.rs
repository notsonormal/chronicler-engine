//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset templates

use askama::Template;

use crate::adapters::driving::http::builders::presets::preset_card_view;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;

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
                <div class="inline-error-slot" data-error-slot="preset-add-system" hidden></div>
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
                <div class="inline-error-slot" data-error-slot="preset-add-quantifier" hidden></div>
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
                <div class="inline-error-slot" data-error-slot="preset-add-impersonate" hidden></div>
                <div class="form-actions">
                    <button type="submit" class="btn-primary">Add Preset</button>
                </div>
            </form>
        </details>
    </div>

    <div class="preset-section">
        <h2>Options Prompts</h2>
        <p class="preset-section-desc">These prompts guide the options LLM that generates the pickable next-action suggestions. Activating one sets the settings-level default; a game that chose its own Options preset keeps it.</p>
        {% for card in self.options_cards() %}
        {{ card }}
        {% endfor %}

        <details class="preset-add">
            <summary class="btn-cyan preset-add-toggle">Add Options Prompt Preset</summary>
            <form hx-post="/prompt-presets" hx-target=".prompt-presets-panel" hx-swap="outerHTML">
                <input type="hidden" name="preset_type" value="options" />
                <div class="form-group">
                    <label for="options-preset-name">Name</label>
                    <input type="text" id="options-preset-name" name="name" placeholder="My Custom Options Prompt" required />
                </div>
                <div class="form-group">
                    <label for="options-preset-role">Role</label>
                    <textarea id="options-preset-role" name="role" rows="4" placeholder="Enter role description..."></textarea>
                </div>
                <div class="form-group">
                    <label for="options-preset-instructions">Instructions</label>
                    <textarea id="options-preset-instructions" name="instructions" rows="8" placeholder="Enter instructions..."></textarea>
                </div>
                <div class="form-group">
                    <label for="options-preset-style">Writing Style</label>
                    <textarea id="options-preset-style" name="writing_style" rows="4" placeholder="Enter writing style..."></textarea>
                </div>
                <div class="form-group">
                    <label for="options-preset-output">Output Format</label>
                    <textarea id="options-preset-output" name="output_format" rows="6" placeholder="Enter output format..."></textarea>
                </div>
                <div class="inline-error-slot" data-error-slot="preset-add-options" hidden></div>
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
    pub options_presets: Vec<PromptPreset>,
    /// The card partial owns the active-slot lookup, so the template needs the
    /// settings the panel was rendered from rather than a pre-resolved ids list.
    pub settings: AppSettings,
}

impl PromptPresetsTemplate {
    fn cards(presets: &[PromptPreset], settings: &AppSettings) -> Vec<SafeHtml> {
        presets
            .iter()
            .map(|preset| preset_card_view(preset, settings))
            .collect()
    }

    pub fn system_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.system_presets, &self.settings)
    }

    pub fn quantifier_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.quantifier_presets, &self.settings)
    }

    pub fn impersonate_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.impersonate_presets, &self.settings)
    }

    pub fn options_cards(&self) -> Vec<SafeHtml> {
        Self::cards(&self.options_presets, &self.settings)
    }
}
