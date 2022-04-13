<h1 align="center">Envoy</h1>
<div align="center">
 <strong>
   Serve the web
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/envoy">
    <img src="https://img.shields.io/crates/v/envoy.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/envoy">
    <img src="https://img.shields.io/crates/d/envoy.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/envoy">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/envoy">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/http-rs/envoy/blob/main/.github/CONTRIBUTING.md">
      Contributing
    </a>
    <span> | </span>
    <a href="https://discord.gg/x2gKzst">
      Chat
    </a>
  </h3>
</div>

Envoy is a minimal and pragmatic Rust web application framework built for
rapid development. It comes with a robust set of features that make building
async web applications and APIs easier and more fun.

## Getting started

In order to build a web app in Rust you need an HTTP server, and an async
runtime. After running `cargo init` add the following lines to your
`Cargo.toml` file:

```toml
# Example, use the version numbers you need
envoy = "0.16.0"
async-std = { version = "1.8.0", features = ["attributes"] }
serde = { version = "1.0", features = ["derive"] }
```

## Examples

Create an HTTP server that receives a JSON body, validates it, and responds
with a confirmation message.

```rust
use envoy::Context;
use envoy::prelude::*;

#[derive(Debug, Deserialize)]
struct Animal {
    name: String,
    legs: u16,
}

#[async_std::main]
async fn main() -> envoy::Result<()> {
    let mut app = envoy::new();
    app.at("/orders/shoes").post(order_shoes);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn order_shoes(mut req: Context<()>) -> envoy::Result {
    let Animal { name, legs } = ctx.body_json().await?;
    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}
```

```sh
$ curl localhost:8080/orders/shoes -d '{ "name": "Chashu", "legs": 4 }'
```
```text
Hello, Chashu! I've put in an order for 4 shoes
```

```sh
$ curl localhost:8080/orders/shoes -d '{ "name": "Mary Millipede", "legs": 750 }'
```
```text
Hello, Mary Millipede! I've put in an order for 750 shoes
```

See more examples in the [examples](https://github.com/http-rs/envoy/tree/main/examples) directory.

## Envoy's design:
- [Rising Envoy: building a modular web framework in the open](https://rustasync.github.io/team/2018/09/11/envoy.html)
- [Routing and extraction in Envoy: a first sketch](https://rustasync.github.io/team/2018/10/16/envoy-routing.html)
- [Middleware in Envoy](https://rustasync.github.io/team/2018/11/07/envoy-middleware.html)
- [Envoy's evolving middleware approach](https://rustasync.github.io/team/2018/11/27/envoy-middleware-evolution.html)
- [Envoy, the present and future of](https://blog.yoshuawuyts.com/envoy/)
- [Envoy channels](https://blog.yoshuawuyts.com/envoy-channels/)

## Community Resources
<sub>To add a link to this list, [edit the markdown
file](https://github.com/http-rs/envoy/edit/main/README.md) and
submit a pull request (github login required)</sub><br/><sup>Listing here
does not constitute an endorsement or recommendation from the envoy
team. Use at your own risk.</sup>

### Listeners
* [envoy-rustls](https://github.com/http-rs/envoy-rustls) TLS for envoy based on async-rustls
* [envoy-acme](https://github.com/http-rs/envoy-acme) HTTPS for envoy with automatic certificates, via Let's Encrypt and ACME tls-alpn-01 challenges

### Template engines
* [envoy-tera](https://github.com/jbr/envoy-tera)
* [envoy-handlebars](https://github.com/No9/envoy-handlebars)
* [askama](https://github.com/djc/askama) (includes support for envoy)

### Routers
* [envoy-fluent-routes](https://github.com/mendelt/envoy-fluent-routes)

### Auth
* [envoy-http-auth](https://github.com/chrisdickinson/envoy-http-auth)
* [envoy-openidconnect](https://github.com/malyn/envoy-openidconnect)

### Testing
* [envoy-testing](https://github.com/jbr/envoy-testing)

### Middleware
* [envoy-compress](https://github.com/Fishrock123/envoy-compress)
* [envoy-sqlx](https://github.com/eaze/envoy-sqlx) - _SQLx pooled connections & transactions_
* [envoy-trace](https://github.com/no9/envoy-trace)
* [envoy-tracing](https://github.com/ethanboxx/envoy-tracing)
* [opentelemetry-envoy](https://github.com/asaaki/opentelemetry-envoy)
* [driftwood](https://github.com/jbr/driftwood) http logging middleware
* [envoy-compressed-sse](https://github.com/Yarn/envoy_compressed_sse)
* [envoy-websockets](https://github.com/http-rs/envoy-websockets)
* [envoy-csrf](https://github.com/malyn/envoy-csrf)

### Session Stores
* [async-redis-session](https://github.com/jbr/async-redis-session)
* [async-sqlx-session](https://github.com/jbr/async-sqlx-session) (sqlite, mysql, postgres, ...)
* [async-mongodb-session](https://github.com/yoshuawuyts/async-mongodb-session/)

### Example applications
* [dot dot vote](https://github.com/rtyler/dotdotvote/)
* [envoy-example](https://github.com/jbr/envoy-example) (sqlx + askama)
* [playground-envoy-mongodb](https://github.com/yoshuawuyts/playground-envoy-mongodb)
* [envoy-morth-example](https://github.com/No9/envoy-morth-example/)
* [broker](https://github.com/apibillme/broker/) (backend as a service)
* [envoy-basic-crud](https://github.com/pepoviola/envoy-basic-crud) (sqlx + tera)
* [envoy-graphql-mongodb](https://github.com/zzy/envoy-graphql-mongodb)
  - Clean boilerplate for graphql services using envoy, rhai, async-graphql, surf, graphql-client, handlebars-rust, jsonwebtoken, and mongodb.
  - Graphql Services: User register, Salt and hash a password with PBKDF2 , Sign in， JSON web token authentication, Change password， Profile Update, User's query & mutation, and Project's query & mutation.
  - Web Application: Client request, bring & parse GraphQL data, Render data to template engine(handlebars-rust)， Define custom helper with Rhai scripting language.
* [surfer](https://github.com/zzy/surfer)
  - The Blog built on Envoy stack, generated from [envoy-graphql-mongodb](https://github.com/zzy/envoy-graphql-mongodb).
  - Backend for graphql services using envoy, async-graphql, jsonwebtoken, mongodb and so on.
  - Frontend for web application using envoy, rhai, surf, graphql_client, handlebars-rust, cookie and so on.
* [envoy-server-example](https://github.com/Lomect/envoy-server-example)

## Contributing
Want to join us? Check out our [The "Contributing" section of the
guide][contributing] and take a look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

#### Conduct

The Envoy project adheres to the [Contributor Covenant Code of
Conduct](https://github.com/http-rs/envoy/blob/main/.github/CODE_OF_CONDUCT.md).
This describes the minimum behavior expected from all contributors.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[releases]: https://github.com/http-rs/envoy/releases
[contributing]: https://github.com/http-rs/envoy/blob/main/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/http-rs/envoy/labels/good%20first%20issue
[help-wanted]: https://github.com/http-rs/envoy/labels/help%20wanted
