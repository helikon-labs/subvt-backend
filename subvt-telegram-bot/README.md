<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

[![](https://img.shields.io/badge/Kusama-Chat%20on%20Telegram-%23000000)](https://t.me/subvt_kusama_bot)
[![](https://img.shields.io/badge/Polkadot-Chat%20on%20Telegram-E6007A)](https://t.me/subvt_polkadot_bot)

---

# SubVT Telegram Bot

A telegram bot for the validators of the [Polkadot](https://polkadot.network/) and [Kusama](https://kusama.network).
You may find Polkadot bot available for chat at [https://t.me/subvt_polkadot_bot](https://t.me/subvt_polkadot_bot),
and the Kusama bot at [https://t.me/subvt_kusama_bot](https://t.me/subvt_kusama_bot).

This bot is an upgrade of the deprecated [1KV Telegram Bot](https://github.com/helikon-labs/polkadot-kusama-1kv-telegram-bot),
rewritten in Rust, an effort proudly [supported](https://github.com/w3f/Grants-Program/blob/master/applications/subvt-telegram-bot.md)
by the Web3 Foundation [Grants Program](https://github.com/w3f/Grants-Program), and has many new commands, notifications
and reports as documented below.

## Commands

- `/about` - View version and developer information.
- `/add` - Add a new validator to the chat, optionally followed by the stash address.
- `/contact` - Send a bug report or feature request to the dev team.
- `/democracy` - View the referenda being voted and your validators' votes.
- `/help` - View the list of all commands.
- `/networkstatus` - View the current network status information, alias `/network`.
- `/nominations` - View a summary of nominations, alias `/n`.
- `/nominationdetails` - View nomination details, alias `/nd`.
- `/payouts` - View monthly nominator payouts report.
- `/remove` - Remove a validator from the chat.
- `/removeall` - Remove all validators from the chat.
- `/rewards` - View monthly validator rewards (ie income) report.
- `/settings` - Configure notifications.
- `/summary` - View a summary of all your validators.
- `/validatorinfo` - View detailed validator information, alias `/vi`.

## Notifications

All notifications are configurable through the `/settings` command.

- Validator On-Chain Activity
  - 🆘 Offline offence
  - ⭐ New nomination
  - ⬇️ Lost nomination
  - 🥶 Chilled
  - 🚀 Active
  - ⏩🚀 Active next session
  - ⏸ Inactive
  - ⏩⏸ Inactive next session
  - 🥁 Validate intention
  - 💰 Unclaimed payouts
  - ⛓ Block authorship
  - ⚓️ Controller changed
  - 🔑️ Session keys changed
  - 👤 Identity changed
  - 💰 Payout submitted for era
- 1KV (Thousand Validator Programme)
  - 🧬 Binary version change
  - 📈 Rank change
  - 🌏 Location change
  - ✅ Validity change
  - 🟢 Online 🔴 offline status
- Democracy
  - 🗳❌ Cancelled
  - 🗳🔗️ Delegated
  - 🗳🚫 Not passed
  - 🗳✅ Passed
  - 🗳📢 Proposed
  - 🗳✋ Seconded
  - 🗳️▶️ Started
  - 🗳🔗️⏹ Undelegated
  - 🗳 Voted