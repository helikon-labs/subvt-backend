<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# SubVT Telegram Bot Development Report

SubVT Telegram Bot is a rewrite of the 1KV bot for Polkadot and Kusama. Please view the initial
[application](https://github.com/w3f/Grants-Program/blob/master/applications/subvt-telegram-bot.md) for the motivation
behind the development of this bot.

## Solution Overview

<p align="center">
  <a href="https://raw.githubusercontent.com/helikon-labs/subvt/main/document/software/image/01-subvt_system_architecture_large.jpg" target="_blank">
    <img src="https://raw.githubusercontent.com/helikon-labs/subvt/main/document/software/image/01-subvt_system_architecture_x_small.jpg"/>
  </a>
  <br/>
  <i>
    SubVT system (click for a larger version)
  </i>
</p>

Please view the SubVT [system architecture document](https://github.com/helikon-labs/subvt/blob/main/document/software/01-subvt_system_architecture.md)
for details on the separate components of the solution model. SubVT Telegram Bot utilizes the following components of
the SubVT backend system:

- Validator details data including account, identity, active nominations, active stake, 1KV enrollment info and more
get fetched from the Redis instance, which is updated by the SubVT Validator List Updater component.
- Network status data is also fetched from the Redis instance, and this data is updated by the SubVT Network Status
Updater component.
- Rewards and payouts reports are generated using the data in the PostgreSQL instance. This data is created by the
SubVT Block Processor component.
- Chats and validators are persisted in the PostgreSQL instance.

Responses to the Telegram commands are carried out by the Telegram Bot, and all other non-interactive notifications are
created by the SubVT Notification Generator component depending on the initial notification rules created by the
Telegram Bot and the later modifications made to these rules by the user through the `/settings` command on the chat.

Notifications are persisted in the SubVT application PostgreSQL instance, and processed by the
SubVT Notification Processor component. This component is responsible for delivering notifications to various channels
such as push notification for Apple and Google devices, email, SMS, phone calls and Telegram messages.

## Comparison of the Initial Model and the Final Solution

Below is the proposed model in the initial [application](https://github.com/w3f/Grants-Program/blob/master/applications/subvt-telegram-bot.md)
document:

<p align="center">
  <img width="750" src="https://raw.githubusercontent.com/helikon-labs/Grants-Program/subvt-telegram-bot/applications/subvt-telegram-bot-files/02-details_01.jpg"/>
</p>

Although being very close to the final solution, there are two differences between this model and the current
working model:

1. The bot doesn't directly communicate with the validator details server, validator list server, live network status
server and the report service, but fetches the data provided by these services directly from the Redis and PostgreSQL
instances, as shown earlier in the architecture diagram.
2. Notifications are not handled by the bot component directly, but the notification generator and processor components.

## Possible Upgrades

- Era points reports
- Paravalidation reports
  - Votes
  - Points
  - Parachains validated
  - Paravalidation ratio
- Telemetry reports and notifications
  - Online/offline status (currently relies on 1KV data)
  - Bandwidth and peer count reports & alerts
  - Block height alerts

These upgrades are going to be made possible with the further development of the SubVT backend.