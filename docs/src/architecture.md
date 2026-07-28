# Architecture SOTA (State of the Art)

LightX is designed with a zero-cost abstraction philosophy, bypassing runtime route parsing in favor of full macro execution at compile time.

## Genèse de la Performance

En concevant le pipeline "Shift-Left", le code n'est plus instancié par l'humain mais analysé statiquement depuis le modèle TOML pour concevoir un ast strict embarqué avec une résilience aux paniques mémoire.

<div class="mermaid">
sequenceDiagram
    participant Dev as Developer
    participant DB as MySQL DB
    participant Gen as lightx_build
    participant Comp as Rust Compiler
    participant HTTP as LightX Server (Hyper)

    Note over Dev,Gen: Shift-Left Generation Phase
    Dev->>DB: Design SQL schemas
    Dev->>Gen: cargo build
    Gen->>DB: Introspect Structure
    Gen-->>Gen: Generate Static Routes O(1)
    Gen-->>Comp: Emits AOP Handlers (Rust) to OUT_DIR

    Note over Comp,HTTP: Compile & Execution Phase
    Comp->>HTTP: Compile static matrix
    HTTP-->>Dev: Binary Zero-Allocation ready!

</div>
