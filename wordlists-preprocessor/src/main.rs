use std::{error::Error, str::FromStr};
use tokio::{fs, io::AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut dir = fs::read_dir("wordlists").await?;
    while let Some(wordlist_path) = dir.next_entry().await? {
        if wordlist_path
            .file_name()
            .into_string()
            .expect("failed to convert os string to string")
            .ends_with("-words.txt")
        {
            println!("processing {:?}", wordlist_path.file_name());

            let mut path = wordlist_path.path();
            path.set_extension("pre.txt");
            let mut file = fs::File::create(path).await?;

            let mut special_chars = vec![];

            for line in fs::read_to_string(wordlist_path.path()).await?.lines() {
                let split_line: Vec<&str> = line.split_ascii_whitespace().collect();
                if split_line.len() == 3 {
                    let id = u32::from_str(split_line.first().expect("failed to parse id"))?;
                    let word = split_line.get(1).expect("failed to parse word");
                    let occurrences =
                        u32::from_str(split_line.get(2).expect("failed to parse occurrences"))?;

                    if id <= 100 {
                        special_chars.push(word.to_string());
                    } else if occurrences >= 100
                        && !word.contains(|c: char| c.is_numeric())
                        && special_chars.iter().all(|s| !word.contains(s))
                    {
                        file.write_all(word.as_bytes()).await?;
                        file.write_all(b"\n").await?;
                    }
                }
            }

            println!("processed {:?}", wordlist_path.file_name());
        }
    }
    Ok(())
}
