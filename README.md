### Database setup and migrations:

To initialize PostgreSQL and run migrations
```sh
./scripts/init_db.sh
```

If the elephant is already running
```sh
SKIP_DOCKER=true ./scripts/init_db.sh
```

### Run

```sh
RUST_LOG=trace cargo watch -x check -x test -x run | bunyan
```

### Test with tracing

```sh
TEST_LOG=true cargo test | bunyan
```

### Notes on Email Service Provider

In the book, the author used Postmark as the email API provider. For this project, I've chosen to use Amazon Simple Email Service 
(Amazon SES) by integrating it with the AWS SDK.
After creating an AWS account and setting up the identity (email address) from which emails will be sent, it's important to also 
create identities for the email addresses to which you will be sending emails. This is necessary because, by default, unpaid AWS accounts are in 
sandbox mode. In sandbox mode, only verified email addresses can send and receive emails. For more information, check out this [Stack Overflow post](https://stackoverflow.com/questions/37528301/email-address-is-not-verified-aws-ses)
