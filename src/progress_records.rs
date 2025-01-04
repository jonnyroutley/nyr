use sqlx::FromRow;
use iocraft::prelude::*;

#[derive(Clone, FromRow, Debug)]
pub struct ProgressRecord {
    id: i64,
    target_id: i64,
    entry_date: chrono::NaiveDate,
    value: f64,
}

#[derive(Default, Props)]
pub struct ProgressRecordsTableProps<'a> {
    pub progress_records: Option<&'a Vec<ProgressRecord>>,
    pub title: &'a str,
}

#[component]
pub fn ProgressRecordsTable<'a>(
    props: &ProgressRecordsTableProps<'a>
) -> impl Into<AnyElement<'a>> {
    element! {
        View(
            margin_top: 1,
            margin_bottom: 1,
            flex_direction: FlexDirection::Column,
            width: 100,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(width: 100pct, justify_content: JustifyContent::Center, margin_bottom:1, ) {
                Text(content: props.title, weight: Weight::Bold )
            }

            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                View(width: 10pct, justify_content: JustifyContent::Center) {
                    Text(content: "id", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 40pct, justify_content: JustifyContent::Center) {
                    Text(content: "target_id", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 25pct, justify_content: JustifyContent::Center) {
                    Text(content: "entry date", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 25pct, justify_content: JustifyContent::Center) {
                    Text(content: "value", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(props.progress_records.map(|progress_records| progress_records.iter().enumerate().map(|(i, progress_record)| element! {
                View(background_color: if i % 2 == 0 { None } else { Some(Color::DarkGrey) }) {
                    View(width: 10pct, justify_content: JustifyContent::Center) {
                        Text(content: progress_record.id.to_string())
                    }

                    View(width: 40pct, justify_content: JustifyContent::Center) {
                        Text(content: progress_record.target_id.to_string())
                    }

                    View(width: 25pct, justify_content: JustifyContent::Center) {
                        Text(content: progress_record.entry_date.to_string())
                    }
                    View(width: 25pct, justify_content: JustifyContent::Center) {
                        Text(content: progress_record.value.to_string())
                    }
                }
            })).into_iter().flatten())
        }
    }
}
