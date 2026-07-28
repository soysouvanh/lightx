#  LightX Internationalization (i18n)

Welcome to the Zero-Overhead translation directory!

This architectural repository stores static TOML translation pipelines precisely mapped to the underlying system constraints and business logic components.

##  Scale-Resistant Structure
To guarantee limitless horizontal scalability without Git "merge hells", this namespace aggressively mimics the database topological layout (allocating strictly one translation definition per column/handler).

- `schema.toml`: Global fallback error structures injected during compilation (e.g. integer bounds checking).
- `handlers/`: A human-defined dictionary exclusively for mapping backend application flow failures (e.g., `not_found`).
- `overrides/`: Exact matching namespace resolving highly specific backend custom rules to localized strings.

In this architecture, language logic is offloaded entirely to the Frontend. The backend solely resolves static translation keys in `O(1)` runtime operations.