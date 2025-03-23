use std::{error::Error, time::Duration};

use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use scraper::{Html, Selector};
use tokio::time::sleep;

pub struct Rating {
    rating: f32,
    review_count: u32,
}

pub async fn get_ratings_by_google_maps_link(google_maps_link: &str) -> Option<Rating> {
    return match scrape_page(google_maps_link).await {
        Ok(html) => {
            if let Some((rating, review_count)) = extract_review_data(&html).await {
                Some(Rating {
                    rating,
                    review_count,
                })
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            None
        }
    };
}

async fn extract_review_data(html: &str) -> Option<(f32, u32)> {
    let document = Html::parse_document(html);

    let rating_selector = Selector::parse(".fontDisplayLarge").unwrap();
    let rating = document
        .select(&rating_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|e| e.replace(",", "."))
        .map(|el| el.parse::<f32>());

    let review_count_selector = Selector::parse("button.GQjSyb span").unwrap();
    let review_count = document
        .select(&review_count_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|e| e.replace(",", "."))
        .map(|el| el.split_whitespace().collect::<Vec<_>>()[0].parse::<u32>());

    return if let (Some(Ok(rating)), Some(Ok(review_count))) = (rating, review_count) {
        Some((rating, review_count))
    } else {
        None
    };
}

async fn scrape_page(url: &str) -> Result<String, Box<dyn Error>> {
    let config = BrowserConfig::builder()
        .chrome_executable(
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        )
        .no_sandbox()
        .build()?;

    // Launch browser
    let (mut browser, mut handler) = Browser::launch(config).await?;

    // Keep event loop running to avoid crashes
    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // Open a new tab
    let page = browser.new_page(url).await?;
    println!("🚀 Page loaded");

    // Wait for the cookie pop-up to appear
    sleep(Duration::from_secs(5)).await;

    // Try to find and click the "Accept All" button
    let accept_button_xpath = "//span[@jsname='V67aGc']/ancestor::button";
    if let Some(accept_button) = page.find_element(accept_button_xpath).await.ok() {
        println!("🍪 Found 'Accept All' button. Clicking...");
        accept_button.click().await.ok();
        sleep(Duration::from_secs(3)).await; // Wait for page to reload
    } else {
        println!("❌ 'Accept All' button not found.");
    }

    // Scrape the **actual** page content after accepting cookies
    let html = page.content().await?;

    // Close browser
    browser.close().await?;

    Ok(html)
}

pub async fn insert_rating
