# Nostr Paywall Example

This is an example Nostr rust project to enable `402 Payment Required` responses for requests to paid content. To prove payment, a [NIP 98 - HTTP Auth](https://github.com/nostr-protocol/nips/blob/af4cbfbddb2900b7bc4a56b57430989e8b613006/98.md) Nostr event should be included, which will unlock access the content post-payment - ideally on on refresh on successful payment.

This project doesn't handle money, and is a reference project to help you get started. If there is enough interest, it could become a rust create (library) project.

## Unsupported Today (however, ideally on the roadmap)

1. Generating invoices for `402 Payment Required` responses to enable payment (embedding the pubkey and content_id metadata in an invoice)
3. Database schema
4. Database access check logic / query (with access to pubkey and content_id)
5. Actual Content delivery
6. Split payments and similar

## Customisation

1. [check_pubkey_access() in db.rs](src/db.rs) - You can customise the Postgres DB query used to check access. Currently the check returns true for all valid `NIP 98` header.
2. [content in routes.rs](src/routes.rs) - You can write your own content delivery logic here; for URL redirection, Images, proxied content, or anything else.


## Getting Started

```bash
git clone https://github.com/blakejakopovic/nostr_paywall_example
cd nostr_paywall_example

cp .env.example .env
nano .env
# Ensure you update AUTH_EVENT_HOST to match your expected server hostname, or the NIP-98 auth will fail

RUST_LOG=info cargo run --release
```

## Usage / Testing

1. Create a `NIP 98 HTTP Auth event`. You can use this tool with a test pubkey [https://nostrtool.com](https://nostrtool.com). Just keep in mind by default you only have 60 seconds before the event's created_at value becomes too old (and you will get a Bad Request response).

```JSON
{
  "id": "cddeb723c58a2ff78cd5f0f315d855306ce7eada1714a26572ac0f0d2e619925",
  "pubkey": "970e0ebe27a552be8982047974f60302caab0fcb9aa86855f81102e67e084e45",
  "created_at": 1682772206,
  "kind": 27235,
  "tags": [
    [
      "u",
      "http://localhost:8080/m/test_content_id"
    ],
    [
      "method",
      "GET"
    ]
  ],
  "content": "",
  "sig": "2ef81b6b7d494d57dc06ac261aeb1e9c7dfeeb3868b2c4daee604ccdf4cb9cda8f3df08c50f93c99e327ad8fb40175907d47f7dfc100633af3fe38f085d6c888"
}
```

2. Base64 encode the event. Easily done here [https://www.base64encode.org](https://www.base64encode.org).

```
ewogICJpZCI6ICJjZGRlYjcyM2M1OGEyZmY3OGNkNWYwZjMxNWQ4NTUzMDZjZTdlYWRhMTcxNGEyNjU3MmFjMGYwZDJlNjE5OTI1IiwKICAicHVia2V5IjogIjk3MGUwZWJlMjdhNTUyYmU4OTgyMDQ3OTc0ZjYwMzAyY2FhYjBmY2I5YWE4Njg1NWY4MTEwMmU2N2UwODRlNDUiLAogICJjcmVhdGVkX2F0IjogMTY4Mjc3MjIwNiwKICAia2luZCI6IDI3MjM1LAogICJ0YWdzIjogWwogICAgWwogICAgICAidSIsCiAgICAgICJodHRwOi8vbG9jYWxob3N0OjgwODAvbS90ZXN0X2NvbnRlbnRfaWQiCiAgICBdLAogICAgWwogICAgICAibWV0aG9kIiwKICAgICAgIkdFVCIKICAgIF0KICBdLAogICJjb250ZW50IjogIiIsCiAgInNpZyI6ICIyZWY4MWI2YjdkNDk0ZDU3ZGMwNmFjMjYxYWViMWU5YzdkZmVlYjM4NjhiMmM0ZGFlZTYwNGNjZGY0Y2I5Y2RhOGYzZGYwOGM1MGY5M2M5OWUzMjdhZDhmYjQwMTc1OTA3ZDQ3ZjdkZmMxMDA2MzNhZjNmZTM4ZjA4NWQ2Yzg4OCIKfQ==
```

3. Run the following command with your matching event u tag and base64 event output.
```bash
curl -H "AUTHORIZATION: nostr REPLACE_WITH_BASE64_ENCODED_EVENT" localhost:8080/m/test_content_id

```
