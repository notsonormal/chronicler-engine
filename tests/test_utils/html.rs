//! HTML slicing helpers for panel markup without element ids — locate a region by a stable anchor string.

/// Slice one `<div class="preset-card">…</div>` region out of a rendered
/// panel: find `anchor` (a string unique to the card — its duplicate URL or
/// its card title), rfind the card's opening tag before it, and bound the
/// slice at the next card's opening tag (or EOF). `None` when `anchor` is
/// absent.
///
/// Cards deliberately carry no `data-id` (see STRATEGY.md "Stub tier's
/// accepted tax"), so the anchor + sibling-bound algorithm is the shared
/// shape; callers keep their own anchor logic and assertions.
pub fn preset_card_html_slice<'a>(body: &'a str, anchor: &str) -> Option<&'a str> {
    let anchor_pos = body.find(anchor)?;
    let start = body[..anchor_pos]
        .rfind("<div class=\"preset-card")
        .unwrap_or(0);
    let end = body[anchor_pos..]
        .find("<div class=\"preset-card")
        .map(|offset| anchor_pos + offset)
        .unwrap_or(body.len());
    Some(&body[start..end])
}
