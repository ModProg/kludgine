extern crate kludgine;
use kludgine::prelude::*;

fn main() {
    SingleWindowApplication::run(UIExample { image: None });
}

struct UIExample {
    image: Option<Entity<Image>>,
}

impl WindowCreator<UIExample> for UIExample {
    fn window_title() -> String {
        "User Interface - Kludgine".to_owned()
    }
}

impl Window for UIExample {}

#[async_trait]
impl Component for UIExample {
    type Message = ();

    async fn initialize(&mut self, context: &mut Context) -> KludgineResult<()> {
        let sprite = include_aseprite_sprite!("assets/stickguy").await?;
        self.image = Some(
            context
                .new_entity(Image::new(sprite))
                .layout(Layout {
                    location: Point::new(32., 32.),
                    ..Default::default()
                })
                .insert()
                .await?,
        );
        Ok(())
    }

    // async fn update(&mut self, _context: &mut Context, _scene: &SceneTarget) -> KludgineResult<()> {
    //     self.ui.update(scene).await
    // }

    async fn render(
        &self,
        _context: &mut Context,
        _scene: &SceneTarget,
        _location: Rect,
        _effective_style: &EffectiveStyle,
    ) -> KludgineResult<()> {
        // self.ui.render(scene).await?;

        Ok(())
    }

    async fn process_input(
        &mut self,
        _context: &mut Context,
        event: InputEvent,
    ) -> KludgineResult<()> {
        if let Event::MouseButton { .. } = event.event {
            UIExample::open(UIExample { image: None }).await;
        }
        Ok(())
    }
}

// #[derive(Debug)]
// struct Interface {
//     click_count: i32,
// }

// #[async_trait]
// impl Controller for Interface {
//     async fn view(&self) -> KludgineResult<KludgineHandle<Box<dyn View>>> {
//         Label::default()
//             .with_value(self.click_count.to_string())
//             .with_style(Style {
//                 font_size: Some(60.0),
//                 color: Some(Color::new(0.0, 0.5, 0.5, 1.0)),
//                 ..Default::default()
//             })
//             .with_hover_style(Style {
//                 color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
//                 ..Default::default()
//             })
//             .with_padding(Surround::uniform(Dimension::Auto))
//             .build()
//     }

//     async fn mouse_button_down(
//         &mut self,
//         _component: &Component,
//         _button: MouseButton,
//         __window_position: Point,
//     ) -> KludgineResult<ComponentEventStatus> {
//         self.click_count += 1;
//         Ok(ComponentEventStatus::rebuild_view_processed())
//     }
// }
