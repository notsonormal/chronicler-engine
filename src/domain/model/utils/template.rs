//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Template placeholder substitution for author-controlled text fields.

use crate::domain::model::template::TemplateVars;

pub fn render_template(text: &str, vars: &TemplateVars) -> String {
    text.replace("{{user}}", &vars.user)
        .replace("{{persona_description}}", &vars.persona_description)
        .replace("{{persona_personality}}", &vars.persona_personality)
        .replace("{{persona_background}}", &vars.persona_background)
        .replace("{{narrative_perspective}}", &vars.narrative_perspective)
        .replace("{{narrative_tense}}", &vars.narrative_tense)
        .replace("{{option_count}}", &vars.option_count)
}
