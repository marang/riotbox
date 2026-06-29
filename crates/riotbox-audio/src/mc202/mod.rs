mod render_types;
mod source_phrase_sound_design;

pub use render_types::{
    Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
    Mc202RenderRouting, Mc202RenderState, Mc202SourcePhraseRenderPlan, render_mc202_buffer,
};

#[cfg(test)]
mod articulation_tests;
#[cfg(test)]
mod source_phrase_tests;
#[cfg(test)]
mod tests;
