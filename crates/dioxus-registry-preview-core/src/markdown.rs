use std::fmt;

use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html,
};

/// Consumer hooks for semantic Markdown output customization.
pub trait MarkdownHooks {
    /// Returns an `id` for a rendered heading, or `None` to leave it unanchored.
    fn heading_anchor(&self, _level: HeadingLevel, _heading: &str) -> Option<String> {
        None
    }

    /// Returns trusted replacement HTML for a fenced or indented code block.
    ///
    /// The renderer does not sanitize this output. Implementations must sanitize untrusted source
    /// or return only HTML they construct safely themselves.
    fn code_block(&self, _language: Option<&str>, _source: &str) -> Option<String> {
        None
    }
}

/// Markdown features and rendering hooks used while loading documentation.
#[non_exhaustive]
pub struct MarkdownOptions<'a> {
    /// Whether strikethrough syntax is enabled.
    pub strikethrough: bool,
    /// Whether table syntax is enabled.
    pub tables: bool,
    /// Optional semantic rendering hooks.
    pub hooks: Option<&'a dyn MarkdownHooks>,
}

impl<'a> MarkdownOptions<'a> {
    /// Enables or disables strikethrough syntax.
    #[must_use]
    pub const fn with_strikethrough(mut self, enabled: bool) -> Self {
        self.strikethrough = enabled;
        self
    }

    /// Enables or disables table syntax.
    #[must_use]
    pub const fn with_tables(mut self, enabled: bool) -> Self {
        self.tables = enabled;
        self
    }

    /// Uses semantic output hooks for this rendering operation.
    #[must_use]
    pub fn with_hooks(mut self, hooks: &'a dyn MarkdownHooks) -> Self {
        self.hooks = Some(hooks);
        self
    }
}

impl fmt::Debug for MarkdownOptions<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarkdownOptions")
            .field("strikethrough", &self.strikethrough)
            .field("tables", &self.tables)
            .field("hooks", &self.hooks.is_some())
            .finish()
    }
}

impl Default for MarkdownOptions<'_> {
    fn default() -> Self {
        Self {
            strikethrough: true,
            tables: true,
            hooks: None,
        }
    }
}

/// Renders Markdown using the configured extensions and semantic hooks.
///
/// Raw HTML in `markdown` and replacement HTML returned by [`MarkdownHooks::code_block`] are
/// emitted without sanitization. Callers must sanitize untrusted input before rendering it.
#[must_use]
pub fn markdown_to_html(markdown: &str, settings: &MarkdownOptions<'_>) -> String {
    let mut options = Options::empty();

    if settings.strikethrough {
        options.insert(Options::ENABLE_STRIKETHROUGH);
    }
    if settings.tables {
        options.insert(Options::ENABLE_TABLES);
    }

    let events = Parser::new_ext(markdown, options).collect::<Vec<_>>();
    let mut rendered_events = Vec::with_capacity(events.len());
    let mut index = 0;

    while index < events.len() {
        if let (Some(hooks), Event::Start(Tag::CodeBlock(kind))) = (settings.hooks, &events[index])
        {
            let end = events[index + 1..]
                .iter()
                .position(|event| matches!(event, Event::End(TagEnd::CodeBlock)))
                .map(|offset| index + offset + 1);
            if let Some(end) = end {
                let source = event_text(&events[index + 1..end]);
                let language = match kind {
                    CodeBlockKind::Indented => None,
                    CodeBlockKind::Fenced(language) if language.is_empty() => None,
                    CodeBlockKind::Fenced(language) => Some(language.as_ref()),
                };
                if let Some(replacement) = hooks.code_block(language, &source) {
                    rendered_events.push(Event::Html(CowStr::from(replacement)));
                    index = end + 1;
                    continue;
                }
            }
        }

        let mut event = events[index].clone();
        if let (Some(hooks), Event::Start(Tag::Heading { level, id, .. })) =
            (settings.hooks, &mut event)
        {
            let end = events[index + 1..]
                .iter()
                .position(|event| matches!(event, Event::End(TagEnd::Heading(_))))
                .map_or(index, |offset| index + offset + 1);
            let heading = event_text(&events[index + 1..end]);
            if let Some(anchor) = hooks.heading_anchor(*level, &heading) {
                *id = Some(CowStr::from(anchor));
            }
        }
        rendered_events.push(event);
        index += 1;
    }

    let mut output = String::new();
    html::push_html(&mut output, rendered_events.into_iter());

    output
}

fn event_text(events: &[Event<'_>]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
                Some(text.as_ref())
            }
            Event::SoftBreak | Event::HardBreak => Some("\n"),
            _ => None,
        })
        .collect()
}
