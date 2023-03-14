use clap::Parser;
use reqwest::{blocking::Client, blocking::ClientBuilder, cookie::Jar, Url};
use std::{collections::HashMap, sync::Arc};

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    cookie: String,

    #[arg(short, long, default_value = "open")]
    kattis_host: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long)]
    problems_dir: String,
}

#[derive(Debug, Clone)]
struct Problem {
    name: String,
    url: Option<String>,
    local: bool,
    online: bool,
}

fn get_online_problems(args: &Args) -> HashMap<String, Problem> {
    println!("Getting online problems...");

    let client = create_client(args);
    let mut problems = HashMap::new();

    let mut page = 0;
    while let Some(page_problems) = get_online_problems_page(args, &client, page) {
        problems.extend(page_problems);
        page += 1;
    }

    println!("Found {} online problems", problems.len());

    problems
}

fn create_client(args: &Args) -> Client {
    let cookie = format!("EduSiteCookie={}", args.cookie);
    let url = format!("https://{}.kattis.com", args.kattis_host)
        .parse::<Url>()
        .unwrap();

    let jar = Arc::new(Jar::default());
    jar.add_cookie_str(&cookie, &url);

    ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(Arc::clone(&jar))
        .build()
        .unwrap()
}

fn get_online_problems_page(
    args: &Args,
    client: &Client,
    page: u32,
) -> Option<HashMap<String, Problem>> {
    println!("Getting online problems page {page}...");

    let url = format!(
        "https://{}.kattis.com/problems?page={}&show_solved=on&show_tried=off&show_untried=off",
        args.kattis_host, page
    )
    .parse::<Url>()
    .unwrap();

    let response = client.get(url).send().unwrap();
    let text = response.text().unwrap();

    let res = parse_online_problems_page(&text);
    println!(
        "Found {} problems on page {}",
        res.as_ref().unwrap().len(),
        page
    );
    res
}

fn parse_online_problems_page(text: &str) -> Option<HashMap<String, Problem>> {
    let mut problems = HashMap::new();
    let document = scraper::Html::parse_document(text);

    let row_selector = scraper::Selector::parse("table.table2>tbody>tr").unwrap();
    let rows = document.select(&row_selector);

    for row in rows {
        let name_a_selector = scraper::Selector::parse("td>a").unwrap();
        let name_a = row.select(&name_a_selector).next().unwrap();
        let name_url = name_a.value().attr("href").unwrap();
        let name = name_url.split('/').last().unwrap();

        let problem = Problem {
            name: name.to_string(),
            url: Some(name_url.to_string()),
            local: false,
            online: true,
        };
        problems.insert(name.to_string(), problem);
    }

    if problems.is_empty() {
        None
    } else {
        Some(problems)
    }
}

fn read_online_problems_file(args: &Args) -> HashMap<String, Problem> {
    println!("Reading online problems file...");
    let mut problems = HashMap::new();

    let contents = std::fs::read_to_string(&args.input_file).unwrap();

    for line in contents.lines() {
        let split = line.split(';').collect::<Vec<_>>();
        let problem = Problem {
            name: split[0].to_string(),
            url: Some(split[1].to_string()),
            local: false,
            online: true,
        };
        problems.insert(split[0].to_string(), problem);
    }

    println!("Found {} online problems", problems.len());

    problems
}

fn dump_problems_file(args: &Args, problems: &HashMap<String, Problem>) {
    println!("Dumping problems file...");

    let mut contents = String::new();

    for problem in problems.values() {
        contents.push_str(&format!(
            "{};{}\n",
            problem.name,
            problem.url.as_ref().unwrap()
        ));
    }

    std::fs::write(&args.output_file, contents).unwrap();

    println!("Dumped problems file");
}

fn get_local_problems(args: &Args, online_problems: &mut HashMap<String, Problem>) {
    println!("Getting local problems...");
    let mut counter = 0;

    let paths = std::fs::read_dir(&args.problems_dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if online_problems.contains_key(name) {
            let problem = online_problems.get_mut(name).unwrap();
            problem.local = true;
        } else {
            let problem = Problem {
                name: name.to_string(),
                url: None,
                local: true,
                online: false,
            };
            online_problems.insert(name.to_string(), problem);
        }

        counter += 1;
    }

    println!("Found {counter} local problems");
}

fn print_status(problems: &HashMap<String, Problem>) {
    let mut local = Vec::new();
    let mut online = Vec::new();

    for problem in problems.values() {
        if problem.local && !problem.online {
            local.push(problem.clone());
        } else if !problem.local && problem.online {
            online.push(problem.clone());
        }
    }

    println!("=======================================");
    println!("Problems that are local but not online:");
    for problem in local {
        println!("{}", problem.name);
    }

    println!("=======================================");
    println!("Problems that are online but not local:");
    for problem in online {
        println!("{}\t\t\t{}", problem.name, problem.url.unwrap());
    }
}

fn main() {
    let args = Args::parse();

    let mut problems;

    if !args.input_file.is_empty() {
        problems = read_online_problems_file(&args);
    } else {
        problems = get_online_problems(&args);
    }

    if !args.output_file.is_empty() {
        dump_problems_file(&args, &problems);
    }

    get_local_problems(&args, &mut problems);
    print_status(&problems);
}
