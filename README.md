### Run

```sh
RUST_LOG=trace cargo watch -x check -x test -x run | bunyan
```

### Notes on Email Service Provider

In the book, the author used Postmark as the email API provider. In this project, I've opted for Amazon Simple Email Service (Amazon SES)
by integrating it with the AWS SDK. This choice was made to bring something new and to leverage Amazon SES’s scalability and robust features
for handling email workflows in production environments.
