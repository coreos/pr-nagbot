# PR Nagbot

This is a simple application written in Rust that fetches all open PRs in repos specified in a config file, and then posts them to a Slack channel.

## Setup

To build this program, install the Rust toolchain and cargo, and install the program by running:

```
cargo install https://github.com/coreos/pr-nagbot.git
```

To run this program, either create a config file in your current directory called `pr-nagbot.yaml` or specify a different location for a config when running the nagbot using the `-c` commandline argument.

To allow the PR Nagbot to interact with GitHub and Slack, you need to set it up for those services as well. To generate a GitHub access token, log into your account and go to https://github.com/settings/tokens. There, you can generate a token with the "repo" scope to allow pr-nagbot to view both public and private repos, and then copy that access token to the config file.

To get a webhook URL for Slack, go to https://api.slack.com/, select "Start Building", and then name your app and specify what Organization you want to add it to. Then, use the left hand bar to go to "Incoming Webhooks", activate the feature and add a new webhook for the channel you want. Finally, use the left hand bar to go to "OAuth and Permissions" and add "incoming-webhook" to the permission scopes.

Once you have both the GitHub access token and the Slack incoming webhook set up in your config file, the app is ready to use.
