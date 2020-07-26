mod arena;
mod component;
mod context;
mod image;
mod label;
mod layout;
mod node;

pub(crate) use self::node::NodeData;
pub use self::{
    component::{
        Callback, Component, EntityBuilder, InteractiveComponent, LayoutConstraints,
        StandaloneComponent,
    },
    context::*,
    image::Image,
    label::Label,
    layout::*,
    node::Node,
};
use crate::{
    math::{Point, Rect, Surround},
    runtime::Runtime,
    scene::SceneTarget,
    style::Style,
    window::{Event, InputEvent},
    KludgineHandle, KludgineResult,
};
use arena::{HierarchicalArena, Index};
use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet, VecDeque};

static UI: OnceCell<HierarchicalArena> = OnceCell::new();

pub(crate) fn global_arena() -> &'static HierarchicalArena {
    UI.get_or_init(HierarchicalArena::default)
}

pub struct UserInterface<C>
where
    C: InteractiveComponent + 'static,
{
    pub(crate) root: Entity<C>,
    focus: Option<Index>,
    active: Option<Index>,
    hover: Option<Index>,
    last_render_order: Vec<Index>,
}

impl<C> UserInterface<C>
where
    C: InteractiveComponent + 'static,
{
    pub async fn new(root: C) -> KludgineResult<Self> {
        let root = Entity::new({
            let node = Node::new::<_, ()>(
                root,
                Style::default(),
                Style::default(),
                Style::default(),
                Style::default(),
                None,
            );

            global_arena().insert(None, node).await
        });

        let ui = Self {
            root,
            focus: None,
            active: None,
            hover: None,
            last_render_order: Default::default(),
        };
        ui.initialize(root).await?;
        Ok(ui)
    }

    pub async fn render(&mut self, scene: &SceneTarget) -> KludgineResult<()> {
        let mut effective_styles = HashMap::new();

        let layouts = {
            let mut computed_styles = HashMap::new();
            let hovered_indicies = self.hovered_indicies().await;
            let mut traverser = global_arena().traverse(self.root).await;
            let mut found_nodes = VecDeque::new();
            while let Some(index) = traverser.next().await {
                let node = global_arena().get(index).await.unwrap();
                let mut node_style = node.style().await;

                if hovered_indicies.contains(&index) {
                    node_style = node.hover_style().await.inherit_from(&node_style);
                }

                let computed_style = match global_arena().parent(index).await {
                    Some(parent_index) => {
                        node_style.inherit_from(computed_styles.get(&parent_index).unwrap())
                    }
                    None => node_style.clone(),
                };
                computed_styles.insert(index, computed_style);
                found_nodes.push_back(index);
            }

            for (index, style) in computed_styles {
                effective_styles.insert(index, style.effective_style(scene).await);
            }

            // Traverse the found nodes starting at the back (leaf nodes) and iterate upwards to update stretch
            let mut layout_solvers = HashMap::new();
            while let Some(index) = found_nodes.pop_back() {
                let node = global_arena().get(index).await.unwrap();
                let effective_style = effective_styles.get(&index).unwrap().clone();
                let mut context = StyledContext::new(index, scene.clone(), effective_style.clone());
                let solver = node.layout(&mut context).await?;
                layout_solvers.insert(index, KludgineHandle::new(solver));
            }

            let layout_data = LayoutEngine::new(
                layout_solvers,
                effective_styles.clone(), // TODO don't really want to clone here
                self.root.index,
            );

            while let Some(index) = layout_data.next_to_layout().await {
                let effective_style = effective_styles.get(&index).unwrap().clone();
                let mut context = LayoutContext::new(
                    index,
                    scene.clone(),
                    effective_style.clone(),
                    layout_data.clone(),
                );
                let computed_layout = match context.layout_for(index).await {
                    Some(layout) => layout,
                    None => Layout {
                        bounds: Rect::sized(Point::default(), scene.size().await),
                        padding: Surround::default(),
                        margin: Surround::default(),
                    },
                };
                context
                    .layout_within(index, &computed_layout.inner_bounds())
                    .await?;
                let node = global_arena().get(index).await.unwrap();
                node.set_layout(computed_layout).await;
            }

            layout_data
        };

        self.last_render_order.clear();
        while let Some(index) = layouts.next_to_render().await {
            if let Some(layout) = layouts.get_layout(&index).await {
                self.last_render_order.push(index);
                let node = global_arena().get(index).await.unwrap();
                let mut context = StyledContext::new(
                    index,
                    scene.clone(),
                    effective_styles.get(&index).unwrap().clone(),
                );
                node.render_background(&mut context, &layout).await?;
                node.render(&mut context, &layout).await?;
            }
        }

        Ok(())
    }

    async fn hovered_indicies(&mut self) -> HashSet<Index> {
        let mut indicies = HashSet::new();
        let mut hovered_index = self.hover;
        while let Some(index) = hovered_index {
            indicies.insert(index);
            hovered_index = global_arena().parent(index).await;
        }
        indicies
    }

    pub async fn update(&mut self, scene: &SceneTarget) -> KludgineResult<()> {
        // Loop twice, once to allow all the pending messages to be exhausted across all
        // nodes. Then after all messages have been processed, trigger the update method
        // for each node.

        let mut traverser = global_arena().traverse(self.root).await;
        while let Some(index) = traverser.next().await {
            let mut context = Context::new(index);
            let node = global_arena().get(index).await.unwrap();

            node.process_pending_events(&mut context).await?;
        }

        let mut traverser = global_arena().traverse(self.root).await;
        while let Some(index) = traverser.next().await {
            let mut context = SceneContext::new(index, scene.clone());
            let node = global_arena().get(index).await.unwrap();

            node.update(&mut context).await?;
        }

        Ok(())
    }

    pub async fn process_input(&mut self, event: InputEvent) -> KludgineResult<()> {
        let mut traverser = global_arena().traverse(self.root).await;
        while let Some(index) = traverser.next().await {
            match event.event {
                Event::MouseMoved { position } => {
                    // Loop in order of top render to bottom render to find where this position is hovering over.
                    self.hover = None;
                    if let Some(position) = position {
                        for &index in self.last_render_order.iter() {
                            if let Some(node) = global_arena().get(index).await {
                                let layout = node.last_layout().await;
                                if layout.bounds_without_margin().contains(position) {
                                    self.hover = Some(index);
                                    break;
                                }
                            }
                        }
                    }
                }
                Event::MouseWheel { delta, touch_phase } => {}
                Event::MouseButton { button, state } => {}
                _ => {}
            }

            let node = global_arena().get(index).await.unwrap();
            let mut context = Context::new(index);
            node.process_input(&mut context, event).await?;
        }

        Ok(())
    }

    async fn initialize(&self, index: impl Into<Index>) -> KludgineResult<()> {
        let index = index.into();
        let node = global_arena().get(index).await.unwrap();

        node.initialize(&mut Context::new(index)).await
    }
}

impl<C> Drop for UserInterface<C>
where
    C: InteractiveComponent + 'static,
{
    fn drop(&mut self) {
        let root = self.root;
        Runtime::spawn(async move {
            global_arena().remove(root).await;
        });
    }
}

#[derive(Debug)]
pub struct Entity<C, O = ()> {
    index: Index,
    _phantom: std::marker::PhantomData<(C, O)>,
}

impl<C, O> Into<Index> for Entity<C, O> {
    fn into(self) -> Index {
        self.index
    }
}

impl<C, O> Entity<C, O> {
    pub fn new(index: Index) -> Self {
        Self {
            index,
            _phantom: Default::default(),
        }
    }
}

impl<C, O> Clone for Entity<C, O> {
    fn clone(&self) -> Self {
        Self::new(self.index)
    }
}

impl<C, O> Copy for Entity<C, O> {}
