use crate::{
    math::{Dimension, Point, Rect, Size, Surround},
    ui::{
        global_arena,
        layout::{Layout, LayoutSolver},
        Index, LayoutContext,
    },
    KludgineError, KludgineResult,
};
use async_trait::async_trait;
use std::collections::HashMap;
#[derive(Default, Debug)]
pub struct AbsoluteLayout {
    children: HashMap<Index, AbsoluteBounds>,
}

impl AbsoluteLayout {
    pub fn child(
        mut self,
        index: impl Into<Index>,
        bounds: AbsoluteBounds,
    ) -> KludgineResult<Self> {
        self.children.insert(index.into(), bounds.validate()?);
        Ok(self)
    }

    fn solve_dimension(
        start: &Dimension,
        end: &Dimension,
        length: &Dimension,
        available_length: f32,
        content_length: f32,
    ) -> (f32, f32, f32) {
        let content_length = length.points().unwrap_or(content_length);

        let mut remaining_length = available_length - content_length;

        if remaining_length < 0. {
            return (0., available_length, 0.);
        }

        let mut auto_measurements = 0;
        if let Some(points) = start.points() {
            remaining_length -= points;
        } else {
            auto_measurements += 1;
        }

        if let Some(points) = end.points() {
            remaining_length -= points;
        } else {
            auto_measurements += 1;
        }

        let effective_side1 = match start {
            Dimension::Auto => remaining_length / auto_measurements as f32,
            Dimension::Points(points) => *points,
        };

        let effective_side2 = match end {
            Dimension::Auto => remaining_length / auto_measurements as f32,
            Dimension::Points(points) => *points,
        };

        remaining_length = available_length - content_length - effective_side1 - effective_side2;

        if remaining_length < -0. {
            // The padding was too much, we have an edge case with not enough information
            // Do we decrease the padding or do we decrease the width?
            // For now, the choice is to decrease the width
            let content_length = available_length - effective_side1 - effective_side2;
            if content_length < 0. {
                // Ok, we really really are in a pickle. At this point, it almost doesn't matter what we do, because the rendered
                // content size is already 0, so we'll just return 0 for the width and divide the sides evenly *shrug*
                (available_length / 2., 0., available_length / 2.)
            } else {
                (effective_side1, content_length, effective_side2)
            }
        } else {
            // If the dimension is auto, increase the width of the content.
            // If the dimension isn't auto, increase the padding
            match length {
                Dimension::Auto => (
                    effective_side1,
                    content_length + remaining_length,
                    effective_side2,
                ),
                Dimension::Points(_) => (
                    effective_side1 + remaining_length / 2.,
                    content_length,
                    effective_side2 + remaining_length / 2.,
                ),
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct AbsoluteBounds {
    pub left: Dimension,
    pub right: Dimension,
    pub top: Dimension,
    pub bottom: Dimension,
    pub width: Dimension,
    pub height: Dimension,
}

impl AbsoluteBounds {
    fn validate(self) -> KludgineResult<Self> {
        if self.left.is_points() && self.right.is_points() && self.width.is_points() {
            Err(KludgineError::AbsoluteBoundsInvalidHorizontal)
        } else if self.top.is_points() && self.bottom.is_points() && self.height.is_points() {
            Err(KludgineError::AbsoluteBoundsInvalidVertical)
        } else {
            Ok(self)
        }
    }
}

#[async_trait]
impl LayoutSolver for AbsoluteLayout {
    async fn layout_within(
        &self,
        bounds: &Rect,
        content_size: &Size,
        context: &mut LayoutContext,
    ) -> KludgineResult<HashMap<Index, Layout>> {
        println!("Absolute Layout solving for {:?}", content_size);
        let mut computed_layouts = HashMap::new();
        for (&index, child_bounds) in self.children.iter() {
            let mut child_context = context.clone_for(index).await;
            let child_content_size = global_arena()
                .get(index)
                .await
                .unwrap()
                .content_size(
                    child_context.styled_context(),
                    &Size::new(Some(content_size.width), Some(content_size.height)),
                )
                .await?;
            println!("Child content size: {:?}", child_content_size);
            let (left, width, right) = Self::solve_dimension(
                &child_bounds.left,
                &child_bounds.right,
                &child_bounds.width,
                bounds.size.width,
                child_content_size.width,
            );
            let (top, height, bottom) = Self::solve_dimension(
                &child_bounds.top,
                &child_bounds.bottom,
                &child_bounds.height,
                bounds.size.height,
                child_content_size.height,
            );

            computed_layouts.insert(
                index,
                Layout {
                    bounds: Rect::sized(
                        bounds.origin + Point::new(left, top),
                        Size::new(width, height),
                    ),
                    padding: Surround {
                        left,
                        top,
                        right,
                        bottom,
                    },
                },
            );
        }

        Ok(computed_layouts)
    }
}
