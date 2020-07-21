use crate::{
    math::{Rect, Size},
    style::{Layout, Style},
    ui::{BaseComponent, Component, Context, SceneContext, StyledContext},
    window::InputEvent,
    KludgineHandle, KludgineResult,
};
use async_trait::async_trait;
use std::any::Any;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub(crate) trait AnyComponent: PendingEventProcessor + BaseComponent {
    fn as_any(&self) -> &dyn Any;
    fn style(&self) -> &'_ Style;
    fn layout(&self) -> &'_ Layout;
}

#[async_trait]
pub(crate) trait PendingEventProcessor {
    async fn process_pending_events(&mut self, context: &mut Context) -> KludgineResult<()>;
}

impl<T: Component + 'static> AnyComponent for NodeData<T> {
    fn as_any(&self) -> &dyn Any {
        &self.component
    }

    fn style(&self) -> &'_ Style {
        &self.style
    }

    fn layout(&self) -> &'_ Layout {
        &self.layout
    }
}

#[async_trait]
impl<T: Component + 'static> PendingEventProcessor for NodeData<T> {
    async fn process_pending_events(&mut self, context: &mut Context) -> KludgineResult<()> {
        while let Ok(message) = self.receiver.try_recv() {
            self.component.receive_message(context, message).await?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct NodeData<T>
where
    T: Component,
{
    component: T,
    pub(crate) style: Style,
    pub(crate) layout: Layout,
    pub(crate) sender: UnboundedSender<T::Message>,
    receiver: UnboundedReceiver<T::Message>,
}

#[async_trait]
impl<T> BaseComponent for NodeData<T>
where
    T: Component,
{
    async fn initialize(&mut self, context: &mut Context) -> KludgineResult<()> {
        self.component.initialize(context).await
    }
    async fn content_size(
        &self,
        context: &mut StyledContext,
        constraints: &Size<Option<f32>>,
    ) -> KludgineResult<Size> {
        self.component.content_size(context, constraints).await
    }

    async fn render(&self, context: &mut StyledContext, location: &Rect) -> KludgineResult<()> {
        self.component.render(context, location).await
    }

    async fn render_background(
        &self,
        context: &mut StyledContext,
        location: &Rect,
    ) -> KludgineResult<()> {
        self.component.render_background(context, location).await
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.component.update(context).await
    }

    async fn process_input(
        &mut self,
        context: &mut Context,
        event: InputEvent,
    ) -> KludgineResult<()> {
        self.component.process_input(context, event).await
    }
}

#[derive(Clone)]
pub struct Node {
    pub(crate) component: KludgineHandle<Box<dyn AnyComponent>>,
}

impl Node {
    pub fn new<T: Component + 'static>(component: T, style: Style, layout: Layout) -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            component: KludgineHandle::new(Box::new(NodeData {
                style,
                layout,
                component,
                sender,
                receiver,
            })),
        }
    }

    pub async fn style(&self) -> Style {
        let component = self.component.read().await;
        component.style().clone()
    }

    pub async fn layout(&self) -> Layout {
        let component = self.component.read().await;
        component.layout().clone()
    }

    pub async fn content_size(
        &self,
        context: &mut StyledContext,
        constraints: &Size<Option<f32>>,
    ) -> KludgineResult<Size> {
        let component = self.component.read().await;
        component.content_size(context, constraints).await
    }

    /// Called once the Window is opened
    pub async fn initialize(&self, context: &mut Context) -> KludgineResult<()> {
        let mut component = self.component.write().await;
        component.initialize(context).await
    }

    pub async fn render(&self, context: &mut StyledContext, location: &Rect) -> KludgineResult<()> {
        let component = self.component.read().await;
        component.render(context, location).await
    }

    pub async fn render_background(
        &self,
        context: &mut StyledContext,
        location: &Rect,
    ) -> KludgineResult<()> {
        let component = self.component.read().await;
        component.render_background(context, location).await
    }

    pub async fn update(&self, context: &mut SceneContext) -> KludgineResult<()> {
        let mut component = self.component.write().await;
        component.update(context).await
    }

    pub async fn process_input(
        &self,
        context: &mut Context,
        event: InputEvent,
    ) -> KludgineResult<()> {
        let mut component = self.component.write().await;
        component.process_input(context, event).await
    }

    pub async fn process_pending_events(&self, context: &mut Context) -> KludgineResult<()> {
        let mut component = self.component.write().await;
        component.process_pending_events(context).await
    }
}
