use crate::analyze::{SprintsAnalyzed, UserDataAnalyzed};
use crate::model::{Sprint, User};
use itertools::Itertools;
use markdown_builder::Markdown;
use std::fs;
use std::io::Write;
use markdown_table::{Heading, HeadingAlignment, MarkdownTable};

pub trait MarkdownReport {
    fn report_create(&self, team: &String);
}

impl MarkdownReport for SprintsAnalyzed {
    fn report_create(&self, team: &String) {
        let mut doc = Markdown::new();

        doc.h1("Спринты");
        for (sprint, data) in self {
            let data = data
                .iter()
                .filter(|(u, d)| u.teams.contains(&team))
                .collect::<Vec<_>>();
            doc.add_sprint(sprint, data);
        }

        fs::write(format!("{team}.md"), doc.render()).unwrap();
    }
}

trait MarkdownExt {
    fn add_sprint(&mut self, sprint: &Sprint, data: Vec<&(User, UserDataAnalyzed)>);
}

impl MarkdownExt for Markdown {
    fn add_sprint(&mut self, sprint: &Sprint, data: Vec<&(User, UserDataAnalyzed)>) {
        self.h2(format!(
            "{} ({} - {})",
            sprint.name,
            sprint.since.format("%d.%m.%Y"),
            sprint.until.format("%d.%m.%Y"),
        ));

        let mut table = vec![];
        let row = data
            .iter()
            .map(|(u, _)| u)
            .map(|user| user.avatar_url.clone())
            .map(|s| format!("![]({s} =120x)"))
            .map(|s| Heading::new(s, Some(HeadingAlignment::Center)))
            .collect::<Vec<_>>();
        let header = [vec![Heading::new("".to_string(), None)], row].concat();

        let row = data
            .iter()
            .map(|(u, _)| u)
            .map(|user| user.username.clone())
            .map(|s| format!("**{s}**"))
            .collect::<Vec<_>>();
        table.push([vec!["".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(u, _)| u)
            .map(|user| user.role.clone())
            .map(|s| format!("*{s}*"))
            .collect::<Vec<_>>();
        table.push([vec!["".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.commits.clone())
            .map(|c| {
                format!(
                    "**{}** (*+ {}* / *- {}*)",
                    c.change_lines, c.insertions, c.deletions
                )
            })
            .collect::<Vec<_>>();
        table.push([vec!["Вклад в кодовую базу".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.commits.commits.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Сделал коммитов".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.create_pull_requests.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Создал PR".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.merged_pull_requests.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Слил PR".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.closed_pull_requests.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Удалил PR".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.received_discussions.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Получил дискуссий".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.approver_assigned.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Был назначен ревьювером".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.approver_conducted.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Провел ревью".to_string()], row].concat());

        let row = data
            .iter()
            .map(|(_, data)| data)
            .map(|data| data.pull_requests.approver_added_discussions.clone())
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>();
        table.push([vec!["Завел дисскуссий".to_string()], row].concat());

        let mut md_table = MarkdownTable::new(table);
        md_table.with_headings(header);

        self.paragraph(md_table.as_markdown().unwrap());
    }
}
