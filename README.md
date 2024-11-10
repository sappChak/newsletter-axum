### Run

```sh
RUST_LOG=trace cargo watch -x check -x test -x run | bunyan
```

### Test with tracing

```sh
TEST_LOG=true cargo test | bunyan
```


### Notes on Email Service Provider

In the book, the author used Postmark as the email API provider. In this project, I've opted for Amazon Simple Email Service (Amazon SES)
by integrating it with the AWS SDK. 
