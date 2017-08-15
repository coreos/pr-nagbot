// Copyright 2017 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate slack_hook;
extern crate github_rs;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate clap;

use clap::{Arg, App};
use errors::*;
use std::fs::File;
use std::io::Read;
use github_rs::client::Github;
use slack_hook::{Slack, SlackText, SlackTextContent, PayloadBuilder, SlackLink};
use slack_hook::SlackTextContent::{Text, Link};
use std::vec::Vec;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

mod errors {
    use github_rs;
    use slack_hook;
    use serde_json;
    error_chain!{
        foreign_links {
            SlackError(slack_hook::Error);
            GithubError(github_rs::errors::Error);
            JsonError(serde_json::Error);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PR {
    number: usize,
    title: String,
    html_url: String,
}

fn add_prs(repo: &str, prs: Vec<PR>) -> Option<Vec<SlackTextContent>> {
    if prs.is_empty() {
        return None;
    }
    let mut message = vec![Text(SlackText::new(format!("{}:", repo)))];
    for pr in prs {
        message.push(Text(SlackText::new("\n\t- ")));
        message.push(Link(SlackLink::new(
            &pr.html_url,
            &(format!("#{}", pr.number)),
        )));
        message.push(Text(SlackText::new(format!(": {}", pr.title))));
    }
    Some(message)
}

#[derive(Serialize, Deserialize)]
struct Config {
    access_token: String,
    webhook_url: String,
    repos: Vec<Repo>,
}

#[derive(Serialize, Deserialize)]
struct Repo {
    owner: String,
    repo: String,
}

quick_main!(run);

fn run() -> Result<()> {
    let config: Config = {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .arg(
                Arg::with_name("config_file")
                    .short("c")
                    .long("config_file")
                    .takes_value(true)
                    .help("Path to the config file")
                    .default_value("pr-nagbot.yaml"),
            )
            .get_matches();
        let mut config_string = String::new();
        let mut config_file = File::open(matches.value_of("config_file").unwrap())
            .chain_err(|| "Could not open file")?;
        config_file.read_to_string(&mut config_string).chain_err(
            || "Could not read config to string",
        )?;

        serde_yaml::from_str(&config_string).chain_err(
            || "Failed to deserialize config",
        )?
    };
    let client = Github::new(config.access_token)?;
    let slack = Slack::new(config.webhook_url.as_str())?;
    slack.send(&PayloadBuilder::new()
        .text("There are open PRs in these repos:")
        .build()?)?;

    for repo in config.repos {
        let prs = client
            .get()
            .repos()
            .owner(&repo.owner)
            .repo(&repo.repo)
            .pulls()
            .execute();
        match prs {
            Ok((_, _, Some(prs))) => {
                let ds_pr: Vec<PR> = serde_json::from_value(prs)?;
                match add_prs(&repo.repo, ds_pr) {
                    Some(message) => {
                        slack.send(
                            &PayloadBuilder::new().text(message.as_slice()).build()?,
                        )?;
                    }
                    None => {}
                }
            }
            Ok((_, _, None)) => eprintln!("Invalid response from github"),
            Err(e) => eprintln!("Failed to get PRs for {}: {}", repo.repo, e),
        }
    }
    Ok(())
}
