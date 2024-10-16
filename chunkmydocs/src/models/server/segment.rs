use postgres_types::{ FromSql, ToSql };
use serde::{ Deserialize, Serialize };
use strum_macros::{ Display, EnumString };
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BoundingBox {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OCRResult {
    pub bbox: BoundingBox,
    pub text: String,
    pub confidence: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Segment {
    pub segment_id: String,
    pub bbox: BoundingBox,
    pub page_number: u32,
    pub page_width: f32,
    pub page_height: f32,
    pub content: String,
    pub segment_type: SegmentType,
    pub ocr: Option<Vec<OCRResult>>,
    pub image: Option<String>,
    pub html: Option<String>,
    pub markdown: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Chunk {
    pub segments: Vec<Segment>,
    pub chunk_length: i32,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    EnumString,
    Display,
    ToSchema,
    ToSql,
    FromSql
)]
pub enum SegmentType {
    Title,
    #[serde(rename = "Section header")]
    SectionHeader,
    Text,
    #[serde(rename = "List item")]
    ListItem,
    Table,
    Picture,
    Caption,
    Formula,
    Footnote,
    #[serde(rename = "Page header")]
    PageHeader,
    #[serde(rename = "Page footer")]
    PageFooter,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PdlaSegment {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub page_number: u32,
    pub page_width: f32,
    pub page_height: f32,
    pub text: String,
    #[serde(rename = "type")]
    pub segment_type: SegmentType,
}

impl PdlaSegment {
    pub fn to_segment(&self) -> Segment {
        Segment {
            segment_id: Uuid::new_v4().to_string(),
            bbox: BoundingBox {
                left: self.left,
                top: self.top,
                width: self.width,
                height: self.height,
            },
            page_number: self.page_number,
            page_width: self.page_width,
            page_height: self.page_height,
            content: self.text.clone(),
            segment_type: self.segment_type.clone(),
            ocr: None,
            image: None,
            html: None,
            markdown: None,
        }
    }
}

impl Segment {
    fn to_html(&self) -> String {
        match self.segment_type {
            SegmentType::Title => format!("<h1>{}</h1>", self.content),
            SegmentType::SectionHeader => format!("<h2>{}</h2>", self.content),
            SegmentType::Text => format!("<p>{}</p>", self.content),
            SegmentType::ListItem => {
                let parts = self.content.trim().split('.').collect::<Vec<&str>>();
                if parts[0].parse::<i32>().is_ok() {
                    let start_number = parts[0].parse::<i32>().unwrap();
                    let item = parts[1..].join(".").trim().to_string();
                    format!("<ol start='{}'><li>{}</li></ol>", start_number, item)
                } else {
                    let cleaned_content = self.content
                        .trim_start_matches(&['-', '*', '•', '●', ' '][..])
                        .trim();
                    format!("<ul><li>{}</li></ul>", cleaned_content)
                }
            }
            _ =>
                format!(
                    "<span class=\"{}\">{}</span>",
                    self.segment_type.to_string().to_lowercase().replace(" ", "-"),
                    self.content
                ),
        }
    }

    fn to_markdown(&self) -> String {
        match self.segment_type {
            SegmentType::Title => format!("# {}\n\n", self.content),
            SegmentType::SectionHeader => format!("## {}\n\n", self.content),
            SegmentType::ListItem => {
                let parts = self.content.trim().split('.').collect::<Vec<&str>>();
                if parts[0].parse::<i32>().is_ok() {
                    let start_number = parts[0].parse::<i32>().unwrap();
                    let item: String = parts[1..].join(".").trim().to_string();
                    format!("{}. {}", start_number, item)
                } else {
                    let cleaned_content = self.content
                        .trim_start_matches(&['-', '*', '•', '●', ' '][..])
                        .trim();
                    format!("- {}\n\n", cleaned_content)
                }
            }
            SegmentType::Picture => format!("![{}]()", self.segment_id),
            _ => format!("{}\n\n", self.content),
        }
    }

    pub fn finalize(&mut self) {
        self.html = Some(self.to_html());
        self.markdown = Some(self.to_markdown());
    }
}
