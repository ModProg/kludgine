use crate::{
    math::Size,
    scene::SceneTarget,
    style::EffectiveStyle,
    ui::{HierarchicalArena, Index, SceneContext},
    KludgineError, KludgineResult,
};

pub struct StyledContext {
    base: SceneContext,
    effective_style: EffectiveStyle,
}

impl std::ops::Deref for StyledContext {
    type Target = SceneContext;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl StyledContext {
    pub(crate) fn new<I: Into<Index>>(
        index: I,
        scene: SceneTarget,
        effective_style: EffectiveStyle,
        arena: HierarchicalArena,
    ) -> Self {
        Self {
            base: SceneContext::new(index, scene, arena),
            effective_style,
        }
    }

    pub fn clone_for<I: Into<Index>>(&self, index: I) -> Self {
        Self {
            base: self.base.clone_for(index),
            effective_style: self.effective_style.clone(), // TODO this isn't right
        }
    }

    pub fn from_scene_context(effective_style: EffectiveStyle, base: SceneContext) -> Self {
        Self {
            base,
            effective_style,
        }
    }

    pub fn effective_style(&self) -> &'_ EffectiveStyle {
        &self.effective_style
    }

    pub async fn content_size(
        &self,
        index: impl Into<Index>,
        constraints: &Size<Option<f32>>,
    ) -> KludgineResult<Size> {
        let index = index.into();
        let node = self
            .arena
            .get(index)
            .await
            .ok_or(KludgineError::InvalidIndex)?;

        let mut context = self.clone_for(index);
        node.content_size(&mut context, constraints).await
    }
}
