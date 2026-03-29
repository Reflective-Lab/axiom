# Protocol Buffer Definitions

Canonical `.proto` files for all Converge gRPC services.

| File              | Package                | Purpose                                    |
|-------------------|------------------------|--------------------------------------------|
| `converge.proto`  | `converge.v1`          | Main API — bidirectional streaming for mobile/CLI |
| `knowledge.proto` | `converge.knowledge.v1`| Knowledge base — vector search, CRUD, feedback |
| `kernel.proto`    | `converge.llm.v1`      | LLM reasoning kernel — GPU-isolated inference |

Build scripts in individual crates reference this directory via `CARGO_MANIFEST_DIR`.
