This file contains code templates used by code snippets in README.md.
The code from README  `{}` 

```rust,skt-empty-main
{}
fn main() {{}}
```

```rust,skt-connect
use stargate_grpc::*;
#[tokio::main]
async fn main() -> anyhow::Result<()> {{
  {}  
  Ok(())
}}
```

```rust,skt-query
use stargate_grpc::*;
fn main() {{
  {}  
}}
```

```rust,skt-execute
use stargate_grpc::*;
#[tokio::main]
async fn main() -> anyhow::Result<()> {{
  let client: StargateClient = unimplemented!();
  let query: Query = unimplemented!();
  {}  
  Ok(())
}}
```

```rust,skt-result
use stargate_grpc::*;
use std::convert::TryInto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
  let result_set: ResultSet = unimplemented!();
  {}  
  Ok(())
}}
```
