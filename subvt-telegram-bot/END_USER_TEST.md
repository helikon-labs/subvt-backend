<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# End-User Testing the SubVT Telegram Bot

SubVT Telegram Bot's contract is its command list and the list of notifications it provides. Best way to test is to go
through the commands on a running instance and observe notifications either as they happen or by firing the necessary
conditions for them to happen. Below is a list of steps to go through all the user-facing commands.

1. `/start`: Gets run by Telegram when the user first starts the chat.
2. `/about` - View version and developer information.
3. `/networkstatus`: View the network's status.
4. `/add`: Add a validator to the chat. Can be run multiple times.
5. `/democracy`: View open referenda on the network, and the added validators' votes.
6. `/nfts` - View the NFTs owned by a validator's stash account.
7. `/nominations` - View a summary of nominations, alias `/n`.
8. `/nominationdetails` - View nomination details, alias `/nd`.
9. `/payouts` - View monthly nominator payouts report.
10. `/remove` - Remove a validator from the chat.
11. `/rewards` - View monthly validator rewards (ie income) report.
12. `/settings` - Configure notifications.
13. `/summary` - View a summary of all your validators.
14. `/validatorinfo` - View detailed validator information, alias `/vi`.
15. `/contact` - Send a bug report or feature request to the dev team.

Once one or more validators are added to the chat, the SubVT backend is going to start generating notifications. Please
view the [readme](./README.md) for the complete list of notifications.