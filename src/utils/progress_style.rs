use indicatif::ProgressStyle;

const ONLY_MESSAGE_TEMPLATE: &str = "{spinner} {wide_msg}";
const PERCENT_TEMPLATE: &str = "{spinner} {msg:15} {wide_bar:} {pos:>3}/{len:3}%";
const NUMBER_TEMPLATE: &str = "{spinner} {msg:15} {wide_bar:} {pos:>7}/{len}";

pub struct ProgressStyleTemplate;

impl ProgressStyleTemplate {
    pub fn only_message() -> ProgressStyle {
        ProgressStyle::with_template(ONLY_MESSAGE_TEMPLATE).unwrap()
    }

    pub fn percent_bar() -> ProgressStyle {
        ProgressStyle::with_template(PERCENT_TEMPLATE)
            .unwrap()
            .progress_chars("#>-")
    }

    pub fn number_bar() -> ProgressStyle {
        ProgressStyle::with_template(NUMBER_TEMPLATE)
            .unwrap()
            .progress_chars("#>-")
    }
}
