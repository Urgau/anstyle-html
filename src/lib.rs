//! Convert ANSI escape codes to HTML
//!
//! See [`Term`]
//!
//! # Example
//!
//! ```no_run
//! # use anstyle_html::Term;
//! let vte = std::fs::read_to_string("tests/rainbow.vte").unwrap();
//! let html = Term::new().render_html(&vte);
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod adapter;

pub use anstyle_lossy::palette::Palette;
pub use anstyle_lossy::palette::VGA;
pub use anstyle_lossy::palette::WIN10_CONSOLE;

/// Define the terminal-like settings for rendering output
#[derive(Copy, Clone, Debug)]
pub struct Term {
    palette: Palette,
    fg_color: anstyle::Color,
    bg_color: anstyle::Color,
    background: bool,
    font_family: &'static str,
    min_width_px: usize,
}

impl Term {
    /// Default terminal settings
    pub const fn new() -> Self {
        Self {
            palette: WIN10_CONSOLE,
            fg_color: FG_COLOR,
            bg_color: BG_COLOR,
            background: true,
            font_family: "SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace",
            min_width_px: 720,
        }
    }

    /// Select the color palette for [`anstyle::AnsiColor`]
    pub const fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Select the default foreground color
    pub const fn fg_color(mut self, color: anstyle::Color) -> Self {
        self.fg_color = color;
        self
    }

    /// Select the default background color
    pub const fn bg_color(mut self, color: anstyle::Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Toggle default background off with `false`
    pub const fn background(mut self, yes: bool) -> Self {
        self.background = yes;
        self
    }

    /// Minimum width for the text
    pub const fn min_width_px(mut self, px: usize) -> Self {
        self.min_width_px = px;
        self
    }

    /// Render the HTML with the terminal defined
    pub fn render_html(&self, ansi: &str) -> String {
        use std::fmt::Write as _;

        const FG: &str = "fg";
        const BG: &str = "bg";

        let mut styled = adapter::AnsiBytes::new();
        let mut elements = styled.extract_next(ansi.as_bytes()).collect::<Vec<_>>();
        let mut effects_in_use = anstyle::Effects::new();
        for element in &mut elements {
            let style = &mut element.style;
            // Pre-process INVERT to make fg/bg calculations easier
            if style.get_effects().contains(anstyle::Effects::INVERT) {
                *style = style
                    .fg_color(Some(style.get_bg_color().unwrap_or(self.bg_color)))
                    .bg_color(Some(style.get_fg_color().unwrap_or(self.fg_color)))
                    .effects(style.get_effects().remove(anstyle::Effects::INVERT));
            }
            effects_in_use |= style.get_effects();
        }
        let styled_lines = split_lines(&elements);

        let fg_color = rgb_value(self.fg_color, self.palette);
        let bg_color = rgb_value(self.bg_color, self.palette);
        let font_family = self.font_family;

        let line_height = 18;

        let mut buffer = String::new();
        writeln!(&mut buffer, r#"<!DOCTYPE html>"#).unwrap();
        writeln!(&mut buffer, r#"<html>"#).unwrap();
        writeln!(&mut buffer, r#"<head>"#).unwrap();
        writeln!(&mut buffer, r#"  <meta charset="UTF-8">"#).unwrap();
        writeln!(
            &mut buffer,
            r#"  <meta name="viewport" content="width=device-width, initial-scale=1.0">"#
        )
        .unwrap();
        writeln!(
            &mut buffer,
            r#"  <meta http-equiv="X-UA-Compatible" content="ie=edge">"#
        )
        .unwrap();
        writeln!(&mut buffer, r#"  <style>"#).unwrap();
        writeln!(&mut buffer, r#"    .{FG} {{ color: {fg_color} }}"#).unwrap();
        writeln!(&mut buffer, r#"    .{BG} {{ background: {bg_color} }}"#).unwrap();
        for (name, rgb) in color_styles(&elements, self.palette) {
            if name.starts_with(FG_PREFIX) {
                writeln!(&mut buffer, r#"    .{name} {{ color: {rgb} }}"#).unwrap();
            }
            if name.starts_with(BG_PREFIX) {
                writeln!(
                    &mut buffer,
                    r#"    .{name} {{ background: {rgb}; user-select: none; }}"#
                )
                .unwrap();
            }
            if name.starts_with(UNDERLINE_PREFIX) {
                writeln!(
                    &mut buffer,
                    r#"    .{name} {{ text-decoration-line: underline; text-decoration-color: {rgb} }}"#
                )
                .unwrap();
            }
        }
        writeln!(&mut buffer, r#"    .container {{"#).unwrap();
        writeln!(&mut buffer, r#"      line-height: {line_height}px;"#).unwrap();
        writeln!(&mut buffer, r#"    }}"#).unwrap();
        if effects_in_use.contains(anstyle::Effects::BOLD) {
            writeln!(&mut buffer, r#"    .bold {{ font-weight: bold; }}"#).unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::ITALIC) {
            writeln!(&mut buffer, r#"    .italic {{ font-style: italic; }}"#).unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::UNDERLINE) {
            writeln!(
                &mut buffer,
                r#"    .underline {{ text-decoration-line: underline; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::DOUBLE_UNDERLINE) {
            writeln!(
                &mut buffer,
                r#"    .double-underline {{ text-decoration-line: underline; text-decoration-style: double; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::CURLY_UNDERLINE) {
            writeln!(
                &mut buffer,
                r#"    .curly-underline {{ text-decoration-line: underline; text-decoration-style: wavy; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::DOTTED_UNDERLINE) {
            writeln!(
                &mut buffer,
                r#"    .dotted-underline {{ text-decoration-line: underline; text-decoration-style: dotted; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::DASHED_UNDERLINE) {
            writeln!(
                &mut buffer,
                r#"    .dashed-underline {{ text-decoration-line: underline; text-decoration-style: dashed; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::STRIKETHROUGH) {
            writeln!(
                &mut buffer,
                r#"    .strikethrough {{ text-decoration-line: line-through; }}"#
            )
            .unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::DIMMED) {
            writeln!(&mut buffer, r#"    .dimmed {{ opacity: 0.7; }}"#).unwrap();
        }
        if effects_in_use.contains(anstyle::Effects::HIDDEN) {
            writeln!(&mut buffer, r#"    .hidden {{ opacity: 0; }}"#).unwrap();
        }
        writeln!(&mut buffer, r#"    span {{"#).unwrap();
        writeln!(&mut buffer, r#"      font: 14px {font_family};"#).unwrap();
        writeln!(&mut buffer, r#"      white-space: pre;"#).unwrap();
        writeln!(&mut buffer, r#"      line-height: {line_height}px;"#).unwrap();
        writeln!(&mut buffer, r#"    }}"#).unwrap();
        writeln!(&mut buffer, r#"  </style>"#).unwrap();
        writeln!(&mut buffer, r#"</head>"#).unwrap();
        writeln!(&mut buffer).unwrap();

        if !self.background {
            writeln!(&mut buffer, r#"<body>"#).unwrap();
        } else {
            writeln!(&mut buffer, r#"<body class="{BG}">"#).unwrap();
        }
        writeln!(&mut buffer).unwrap();

        writeln!(&mut buffer, r#"  <div class="container {FG}">"#).unwrap();
        for line in &styled_lines {
            if line.iter().any(|e| e.style.get_bg_color().is_some()) {
                for element in line {
                    if element.text.is_empty() {
                        continue;
                    }
                    write_bg_span(&mut buffer, &element.style, &element.text);
                }
                writeln!(&mut buffer, r#"<br />"#).unwrap();
            }

            for element in line {
                if element.text.is_empty() {
                    continue;
                }
                write_fg_span(&mut buffer, element, &element.text);
            }
            writeln!(&mut buffer, r#"<br />"#).unwrap();
        }
        writeln!(&mut buffer, r#"  </div>"#).unwrap();
        writeln!(&mut buffer).unwrap();

        writeln!(&mut buffer, r#"</body>"#).unwrap();
        writeln!(&mut buffer, r#"</html>"#).unwrap();
        buffer
    }
}

const FG_COLOR: anstyle::Color = anstyle::Color::Ansi(anstyle::AnsiColor::White);
const BG_COLOR: anstyle::Color = anstyle::Color::Ansi(anstyle::AnsiColor::Black);

fn write_fg_span(buffer: &mut String, element: &adapter::Element, fragment: &str) {
    use std::fmt::Write as _;
    let style = element.style;
    let fg_color = style.get_fg_color().map(|c| color_name(FG_PREFIX, c));
    let underline_color = style
        .get_underline_color()
        .map(|c| color_name(UNDERLINE_PREFIX, c));
    let effects = style.get_effects();
    let underline = effects.contains(anstyle::Effects::UNDERLINE);
    let double_underline = effects.contains(anstyle::Effects::DOUBLE_UNDERLINE);
    let curly_underline = effects.contains(anstyle::Effects::CURLY_UNDERLINE);
    let dotted_underline = effects.contains(anstyle::Effects::DOTTED_UNDERLINE);
    let dashed_underline = effects.contains(anstyle::Effects::DASHED_UNDERLINE);
    let strikethrough = effects.contains(anstyle::Effects::STRIKETHROUGH);
    // skipping INVERT as that was handled earlier
    let bold = effects.contains(anstyle::Effects::BOLD);
    let italic = effects.contains(anstyle::Effects::ITALIC);
    let dimmed = effects.contains(anstyle::Effects::DIMMED);
    let hidden = effects.contains(anstyle::Effects::HIDDEN);

    let fragment = html_escape::encode_text(fragment);
    let mut classes = Vec::new();
    if let Some(class) = fg_color.as_deref() {
        classes.push(class);
    }
    if let Some(class) = underline_color.as_deref() {
        classes.push(class);
    }
    if underline {
        classes.push("underline");
    }
    if double_underline {
        classes.push("double-underline");
    }
    if curly_underline {
        classes.push("curly-underline");
    }
    if dotted_underline {
        classes.push("dotted-underline");
    }
    if dashed_underline {
        classes.push("dashed-underline");
    }
    if strikethrough {
        classes.push("strikethrough");
    }
    if bold {
        classes.push("bold");
    }
    if italic {
        classes.push("italic");
    }
    if dimmed {
        classes.push("dimmed");
    }
    if hidden {
        classes.push("hidden");
    }

    let mut need_closing_a = false;

    write!(buffer, r#"<span"#).unwrap();
    if !classes.is_empty() {
        let classes = classes.join(" ");
        write!(buffer, r#" class="{classes}""#).unwrap();
    }
    write!(buffer, r#">"#).unwrap();
    if let Some(hyperlink) = &element.url {
        write!(buffer, r#"<a href="{hyperlink}">"#).unwrap();
        need_closing_a = true;
    }
    write!(buffer, "{fragment}").unwrap();
    if need_closing_a {
        write!(buffer, r#"</a>"#).unwrap();
    }
    write!(buffer, r#"</span>"#).unwrap();
}

fn write_bg_span(buffer: &mut String, style: &anstyle::Style, fragment: &str) {
    use std::fmt::Write as _;
    use unicode_width::UnicodeWidthStr;

    let bg_color = style.get_bg_color().map(|c| color_name(BG_PREFIX, c));

    let fill = if bg_color.is_some() { "█" } else { " " };

    let fragment = html_escape::encode_text(fragment);
    let width = fragment.width();
    let fragment = fill.repeat(width);
    let mut classes = Vec::new();
    if let Some(class) = bg_color.as_deref() {
        classes.push(class);
    }
    write!(buffer, r#"<span"#).unwrap();
    if !classes.is_empty() {
        let classes = classes.join(" ");
        write!(buffer, r#" class="{classes}""#).unwrap();
    }
    write!(buffer, r#">"#).unwrap();
    write!(buffer, "{fragment}").unwrap();
    write!(buffer, r#"</span>"#).unwrap();
}

impl Default for Term {
    fn default() -> Self {
        Self::new()
    }
}

const ANSI_NAMES: [&str; 16] = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright-black",
    "bright-red",
    "bright-green",
    "bright-yellow",
    "bright-blue",
    "bright-magenta",
    "bright-cyan",
    "bright-white",
];

fn rgb_value(color: anstyle::Color, palette: Palette) -> String {
    let color = anstyle_lossy::color_to_rgb(color, palette);
    let anstyle::RgbColor(r, g, b) = color;
    format!("#{r:02X}{g:02X}{b:02X}")
}

const FG_PREFIX: &str = "fg";
const BG_PREFIX: &str = "bg";
const UNDERLINE_PREFIX: &str = "underline";

fn color_name(prefix: &str, color: anstyle::Color) -> String {
    match color {
        anstyle::Color::Ansi(color) => {
            let color = anstyle::Ansi256Color::from_ansi(color);
            let index = color.index() as usize;
            let name = ANSI_NAMES[index];
            format!("{prefix}-{name}")
        }
        anstyle::Color::Ansi256(color) => {
            let index = color.index();
            format!("{prefix}-ansi256-{index:03}")
        }
        anstyle::Color::Rgb(color) => {
            let anstyle::RgbColor(r, g, b) = color;
            format!("{prefix}-rgb-{r:02X}{g:02X}{b:02X}")
        }
    }
}

fn color_styles(
    styled: &[adapter::Element],
    palette: Palette,
) -> impl Iterator<Item = (String, String)> {
    let mut colors = std::collections::BTreeMap::new();
    for element in styled {
        let style = element.style;
        if let Some(color) = style.get_fg_color() {
            colors.insert(color_name(FG_PREFIX, color), rgb_value(color, palette));
        }
        if let Some(color) = style.get_bg_color() {
            colors.insert(color_name(BG_PREFIX, color), rgb_value(color, palette));
        }
        if let Some(color) = style.get_underline_color() {
            colors.insert(
                color_name(UNDERLINE_PREFIX, color),
                rgb_value(color, palette),
            );
        }
    }

    colors.into_iter()
}

fn split_lines(styled: &[adapter::Element]) -> Vec<Vec<adapter::Element>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    for mut element in styled.iter().cloned() {
        while let Some((current, remaining)) = element.text.split_once('\n') {
            let current = current.strip_suffix('\r').unwrap_or(current);
            let mut new_element = element.clone();
            new_element.text = current.to_owned();
            current_line.push(new_element);
            lines.push(current_line);
            current_line = Vec::new();
            element.text = remaining.to_owned();
        }
        current_line.push(element);
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
