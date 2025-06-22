use super::{Color, FontSize, FontWeight, GenerateTemplateId, Margin, Spacing, WritingMode};
use crate::entities::{FileId, UserId, Version};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct GenerateTemplate {
    pub id: GenerateTemplateId,
    pub user_id: UserId,
    pub background_image_file_id: FileId,
    pub font_color: Color,
    pub writing_mode: WritingMode,
    pub margin_block_start: Margin,
    pub margin_inline_start: Margin,
    pub line_spacing: Spacing,
    pub letter_spacing: Spacing,
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disabled: bool,
    pub version: Version,
}
