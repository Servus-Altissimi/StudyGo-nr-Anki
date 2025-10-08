//   ___ _           _       ___                     _        _   _
//  / __| |_ _  _ __| |_  _ / __|___   _ _  _ _     /_\  _ _ | |_(_)
//  \__ \  _| || / _` | || | (_ / _ \ | ' \| '_|   / _ \| ' \| / / |
//  |___/\__|\_,_\__,_|\_, |\___\___/ |_||_|_|(_) /_/ \_\_||_|_\_\_|
//                     |__/

// StudyGo, als een dienst, is verschrikkelijk. De website is duidelijk niet ontworpen om
// scholieren te helpen. Het is eerder ontworpen om zinloze extras in hun gezicht duwen.
// Toch is het populaire onder zowel scholieren als docenten.

// Anki is echter een efficiënte manier om flitskaarten goed te memorizeren, en dat te blijven
// doen zonder onnodige extra'. Dit programma maakt de overstap van StudyGo naar Anki
// gemakkelijk, zonder flitskaarten te missen.

// Copyright 2025 Servus Altissimi (Pseudonym)

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::error::Error;

use headless_chrome::{Browser, protocol::cdp::Page};
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Flashcard {
    front: String,
    back: String,
}


fn read_urls_from_file(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err("File not found".into());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let urls: Vec<String> = reader
    .lines()
    .filter_map(|line| line.ok())
    .map(|line| line.trim().to_string())
    .filter(|line| !line.is_empty() && !line.starts_with('#'))
    .collect();

    Ok(urls)
}

fn scrape_flashcards(browser: &Browser, url: &str) -> Result<Vec<Flashcard>, Box<dyn Error>> {
    let tab = browser.new_tab()?;

    tab.navigate_to(url)?;
    tab.wait_for_element(".pair-list-item")?;
    std::thread::sleep(Duration::from_secs(2)); // Rekening houden met netwerk

    let html = tab.get_content()?;
    let document = Html::parse_document(&html);

    // Alle paren juist selecteren:
    let pair_selector = Selector::parse(".pair-list-item").unwrap();
    let info_selector = Selector::parse(".info.notranslate span.show-on-render").unwrap();

    let mut cards = Vec::new();

    for pair in document.select(&pair_selector) {
        let infos: Vec<_> = pair.select(&info_selector).collect();

        if infos.len() >= 2 {
            let front = infos[0].text().collect::<String>().trim().to_string();
            let back = infos[1].text().collect::<String>().trim().to_string();

            if !front.is_empty() && !back.is_empty() {
                cards.push(Flashcard {front, back});
            }
        }
    }

    Ok(cards)
}

fn write_to_csv(cards: &[Flashcard], filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    for card in cards {
        let front = escape_csv(&card.front);
        let back = escape_csv(&card.back);
        writeln!(file, "{},{}", front, back)?;
    }

    Ok(())
}

fn escape_csv(text: &str) -> String {
    if text.contains(',') || text.contains('"') || text.contains('\n') {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("StudyGo nr. Anki!");
    println!("{}", "=".to_string().repeat(64));

    let urls = read_urls_from_file("urls.txt").unwrap_or_else(|_| {
        println!("Geen urls.txt gevonden.");
        eprintln!("Maak een urls.txt aan, meer informatie is te vinden op https://github.com/Servus-Altissimi/StudyGo-nr-Anki.\n");
        Vec::new()
    });

    if urls.is_empty() {
        eprintln!("Heen URLs gevonden");
        return Ok(());
    }

    println!("URLs om te doornemen: {}", urls.len());
    println!("Headless Browser wordt gestart");

    let browser = Browser::default()?; // kritisch

    let mut all_cards = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for (i, url) in urls.iter().enumerate() {
        println!("[{}/{}] Scraping: {}", i + 1, urls.len(), url);
        match scrape_flashcards(&browser, url) {
            Ok(cards) => {
                println!("Gevonden kaarten: {}", cards.len());
                all_cards.extend(cards);
                successful += 1;
            }
            Err(e) => {
                eprintln!("Probleem: {}", e);
                failed += 1;
            }
        }
        println!("\n");
    }

    if !all_cards.is_empty() {
        write_to_csv(&all_cards, "anki.csv")?;
        println!("\ncards.csv is aangemaakt!");
        println!("  Hoeveelheid flitskaarten: {}", all_cards.len());
        println!("  Success: {}", successful);
        if failed > 0 {
            println!("  Problemen: {}", failed);
        }
        println!("\nImporteer cards.csv in Anki om het te gebruiken!");
    } else {
        println!("\nGeen flitskaarten om te exporteren.");
    }

    Ok(())
}

