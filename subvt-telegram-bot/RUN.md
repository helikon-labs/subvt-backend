<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# Running the SubVT Telegram Bot

SubVT backend in production for Polkadot and Kusama combined consists of 34 executables and is run on
[Docker](https://www.docker.com/) using [Docker Compose](https://docs.docker.com/compose/) for convenience purposes.
Follow the steps below to run the SubVT backend on Docker Compose.

1. Make sure you have [Docker](https://www.docker.com/) installed on your system.
2. Get your Telegram bots registered using [BotFather](https://t.me/BotFather). You are going to need two bots if you want to run for both Kusama and Polkadot.
3. Clone this repository `git clone https://github.com/helikon-labs/subvt-backend.git`.
4. Go to the [directory](../_docker/compose) where the Docker Compose files reside `cd subvt-backend/_docker/compose`.
5. Make the helper shell scripts executable by running the command `chmod +x *.sh`.
6. Rename the [.env.sample](../_docker/compose/.env.sample) to `.env`.
7. Edit the `.env` file. Critical ones are:
   1. `KUSAMA_TELEGRAM_API_TOKEN`: Telegram API token for the Kusama bot.
   2. `KUSAMA_TELEGRAM_BOT_USERNAME`: Full Telegram bot username for the Kusama bot.
   3. `POLKADOT_TELEGRAM_API_TOKEN`: Telegram API token for the Polkadot bot.
   4. `POLKADOT_TELEGRAM_BOT_USERNAME`: Full Telegram bot username for the Polkadot bot.
   5. `KUSAMA_RPC_URL`: Kusama node RPC URL. Port number is mandatory, e.g. `wss://kusama-rpc.polkadot.io:443`. SubVT backend can function with the public RPC endpoints as set by the defaults in the `.env.sample` file, but it requires a locally available dedicated archive node to perform well.
   6. `POLKADOT_RPC_URL`: Same as previous, but for Polkadot.
   7. `KUSAMA_BLOCK_PROCESSOR_START_NUMBER`: Start block number for the Kusama block processor. Set this to a recent block if you don't need the historical data for rewards and payouts reports.
   8. `POLKADOT_BLOCK_PROCESSOR_START_NUMBER`: Same as the previous, but for Polkadot.
   9. `FONT_DIR`: Set this to the full path of the [_fonts](../_fonts) directory in the SubVT source root directory.
   10. `TMP_DIR`: Set this to the full path of an arbitrary temporary folder. This folder is going to be used for the temporary storage of the chart image files before they get sent to the Telegram chat they're prepared for.
   11. `PROMETHEUS_DIR`: Set this to the full path of the [_prometheus](../_prometheus) directory in the SubVT source root directory.
   12. `TEMPLATE_DIR`: Set this to the full path of the [_template](../_template) directory in the SubVT source root directory. This directory contains the notification template files.
8. Run the script [up.sh](../_docker/compose/up.sh). You may need to run it with `sudo` privileges on Linux systems. This script is going to:
   1. Fetch the latest images from Docker Hub.
   2. Configure your containers.
   3. Run the whole SubVT backend system for Polkadot and Kusama, including the Telegram bots.
9. Your bots should be available for chat after the successful completion of the previous step.

# Stopping the Bot

Please run the [stop.sh](../_docker/compose/stop.sh) shell script to stop the whole SubVT backend, or [down.sh](../_docker/compose/down.sh) to stop and remove the containers and resources altogether.