use gpui_kit::base::StyledExt as _;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

struct TinytextApp;

impl Render for TinytextApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().border;

        div()
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                div()
                    .h_flex()
                    .flex_none()
                    .h_9()
                    .px_3()
                    .items_center()
                    .border_b_1()
                    .border_color(border)
                    .text_sm()
                    .child("MenuBar & Tabs"),
            )
            .child(
                div()
                    .h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .flex_none()
                            .w_64()
                            .h_full()
                            .p_3()
                            .border_r_1()
                            .border_color(border)
                            .text_sm()
                            .child("File Explorer"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .min_w_0()
                            .p_3()
                            .text_sm()
                            .child("Main Editor View"),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .flex_none()
                    .h_7()
                    .px_3()
                    .items_center()
                    .border_t_1()
                    .border_color(border)
                    .text_xs()
                    .child("Ln 1, Col 1    UTF-8    Plain Text    Saved"),
            )
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Tinytext".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| TinytextApp))
            .expect("failed to open window");
    });
}
