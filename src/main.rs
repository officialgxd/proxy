use actix_web::{get,  App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use scraper::{Html, Selector};
use std::fs;
use std::time::{Duration, Instant};
use teloxide::requests::Requester;
use teloxide::Bot;
use teloxide_core::types::*;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let bot = Bot::new("7015908466:AAGQ74yCkuF_I8_zlrI308Cyhby2ajTLup8");

    let _bot_task = tokio::spawn(async move {
        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            let document = InputFile::file("./proxies.txt");
            bot.send_document(msg.chat.id, document).await?;
            Ok(())
        })
        .await;
    });

    let port = std::env::var("PORT").unwrap_or("4000".to_string());
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run();
   
    let _ = server.await?;

    let mut start = Instant::now();
    let timeout_duration = Duration::from_secs(3600);

    loop {
        if start.elapsed() >= timeout_duration {
            if let Err(err) = fs::remove_file("./proxies.txt") {
                eprintln!("Error deleting file: {}", err);
            }

            let mut tasks = Vec::new();
            // Advanced proxy scraping
            let advanced_proxy_url = "https://advanced.name/freeproxy";
            let advanced_proxy_tasks = scrape_proxies(
                advanced_proxy_url,
                "table#table_proxies tbody tr",
                "td[data-ip]",
                "td[data-port]",
            )
            .await;
            tasks.extend(advanced_proxy_tasks);

            // Free proxy scraping
            let free_proxy_url = "https://free-proxy-list.net/#";
            let free_proxy_tasks =
                scrape_proxies(free_proxy_url, "tr", "td:nth-child(1)", "td:nth-child(2)").await;
            tasks.extend(free_proxy_tasks);

            // Wait for the next timeout before attempting to delete the file again
            start = Instant::now();
        }
    }
    
}

async fn scrape_proxies(
    url: &str,
    tr_selector: &str,
    ip_selector: &str,
    port_selector: &str,
) -> Vec<String> {
    let client = Client::new();
    let mut tasks = Vec::new();

    if let Ok(response) = client.get(url).send().await {
        if let Ok(body) = response.text().await {
            let document = Html::parse_document(&body);
            let ip_selector = Selector::parse(ip_selector).unwrap();
            let port_selector = Selector::parse(port_selector).unwrap();

            for tr in document.select(&Selector::parse(tr_selector).unwrap()) {
                if let (Some(ip), Some(port)) = (
                    tr.select(&ip_selector).next(),
                    tr.select(&port_selector).next(),
                ) {
                    let ip_text = ip.text().collect::<String>();
                    let port_text = port.text().collect::<String>();
                    if ip_text.len() + port_text.len() > 13 {
                        let task = format!("{}:{}", ip_text, port_text);
                        tasks.push(task);
                    }
                }
            }
        }
    }

    // Saving to file
    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Truncate the file to clear its contents
        .open("proxies.txt")
        .await
    {
        for task in &tasks {
            if let Err(_) =
                tokio::io::AsyncWriteExt::write_all(&mut file, format!("{}\n", task).as_bytes())
                    .await
            {
                eprintln!("Error writing proxy to file.");
            }
        }
    }

    tasks
}
