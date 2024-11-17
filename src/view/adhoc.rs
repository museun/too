use super::{Builder, Response, Ui, View, ViewExt};

/// Adhoc views are views built out of other views
///
/// You can create views out of other views, e.g. those created by [`ui.show()`](Ui::show) and [`ui.show_children()`](Ui::show_children)
///
/// These can be 'shown' with [`ui.adhoc()`](Ui::adhoc). All views that do not
/// show children can be shown using this method. But an adhoc view can only be
/// shown with this method, rather than [`ui.show()`](Ui::show)
///
/// ## Example
/// ```rust,no_run
/// // We can create a style for our adhoc view
/// #[derive(Copy, Clone, Debug)]
/// struct ButtonWithLabelStyle {
///     attribute: Option<Attribute>,
/// }
///
/// impl ButtonWithLabelStyle {
///     fn default(_palette: &Palette) -> Self {
///         Self { attribute: None }
///     }
///
///     fn italic(_palette: &Palette) -> Self {
///         Self {
///             attribute: Some(Attribute::ITALIC),
///         }
///     }
/// }
///
/// type ButtonWithLabelClass = fn(&Palette) -> ButtonWithLabelStyle;
///
/// // We can create a builder for our adhoc view
/// struct ButtonWithLabel<'a> {
///     label: &'a str,
///     button: &'a str,
///     class: StyleKind<ButtonWithLabelClass, ButtonWithLabelStyle>,
/// }
///
/// impl<'a> ButtonWithLabel<'a> {
///     const fn class(mut self, class: ButtonWithLabelClass) -> Self {
///         self.class = StyleKind::Deferred(class);
///         self
///     }
///
///     const fn style(mut self, style: ButtonWithLabelStyle) -> Self {
///         self.class = StyleKind::Direct(style);
///         self
///     }
/// }
///
/// impl<'v> Adhoc<'v> for ButtonWithLabel<'v> {
///     type Output = Response<ButtonResponse>;
///
///     fn show(self, ui: &Ui) -> Self::Output {
///         let style = match self.class {
///             StyleKind::Deferred(style) => (style)(&ui.palette()),
///             StyleKind::Direct(style) => style,
///         };
///
///         let attr = style.attribute.unwrap_or(Attribute::RESET);
///
///         // our adhoc view is just a button next to a label
///         ui.horizontal(|ui| {
///             let resp = ui.button(self.button);
///             ui.show(label(self.label).attribute(attr));
///             resp
///         })
///         .into_inner()
///     }
/// }
///
/// // builder for our adhoc view
/// fn button_with_label<'a>(label: &'a str, button: &'a str) -> ButtonWithLabel<'a> {
///     ButtonWithLabel {
///         label,
///         button,
///         class: StyleKind::Deferred(ButtonWithLabelStyle::default),
///     }
/// }
///
/// // and then we can show it:
/// ui.adhoc(button_with_label("hello", "world").class(ButtonWithLabelStyle::italic));
/// ```
pub trait Adhoc<'v>: Sized {
    /// The output of this view
    type Output: 'static;
    /// Show the view, returning its output
    fn show(self, ui: &Ui) -> Self::Output;
}

impl<'v, T> Adhoc<'v> for T
where
    T: Builder<'v>,
{
    type Output = Response<<T::View as View>::Response>;
    fn show(self, ui: &Ui) -> Self::Output {
        <T as ViewExt>::show(self, ui)
    }
}
