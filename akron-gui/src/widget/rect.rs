use iced::{
    Background, Border, Color, Element, Length, Padding, Rectangle, Size, Theme,
    advanced::{Layout, Widget, layout, mouse, renderer, widget::Tree},
};

pub struct Rect<'a, Theme = iced::Theme>
where
    Theme: Catalog,
{
    width: f32,
    height: f32,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Rect<'a, Theme>
where
    Theme: Catalog,
{
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            class: Theme::default(),
        }
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Rect<'_, Theme>
where
    Renderer: renderer::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.width, self.height))
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background.unwrap_or(Color::TRANSPARENT.into()),
        );

        if let Some(style) = style.inner {
            let bounds = bounds.shrink(style.padding);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    ..renderer::Quad::default()
                },
                style.background.unwrap_or(Color::TRANSPARENT.into()),
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Rect<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + renderer::Renderer,
{
    fn from(rect: Rect<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(rect)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub border: Border,
    pub background: Option<Background>,
    pub inner: Option<Inner>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Inner {
    pub border: Border,
    pub background: Option<Background>,
    pub padding: Padding,
}

pub trait Catalog: Sized {
    type Class<'a>;
    fn default<'a>() -> Self::Class<'a>;
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}
