use std::fs::read_to_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = std::env::args().skip(1);

    let file = read_to_string(inputs.next().unwrap())?;
    let doc = compl::read::parse_doc(&file);

    let inputs: Vec<_> = inputs.collect();
    let rule = doc.rule_book.iter().find_map(|r| r.try_rule(&inputs));

    if let Some(active_rule) = rule {
        let words = active_rule.words(&doc);
        for w in words {
            print!("{w} ");
        }
    }
    Ok(())
}
