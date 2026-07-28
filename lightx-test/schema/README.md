#  LightX Single Source of Truth

Welcome to the schema architecture directory!

This directory is strictly **auto-generated** at compile time by the LightX framework through deep database introspection. It acts as the absolute Data Dictionary (Single Source of Truth) for the entire application ecosystem.

##  CRITICAL WARNING
**DO NOT MANUALLY EDIT ANY FILES IN THIS DIRECTORY!**

All structural files (`.toml`) here are regenerated from scratch during the `cargo build` macro expansion phase. Any manual file modifications or additions will be permanently overwritten.

If you need to force custom validation rules or adjust generated types, you must utilize the `../overrides/` directory instead.