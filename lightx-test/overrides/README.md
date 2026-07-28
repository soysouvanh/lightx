#  LightX Overrides Strategy

Welcome to the overrides directory!

LightX's "Database-First" philosophy completely overwrites the `schema/` directory on every compilation. 
**Never modify the files in `schema/` as they will be overwritten!**

If you want to override business validation rules (e.g., forcing `min_length = 5` on an SQL column) or inject custom metadata, you must reproduce the exact table topology here.

##  Example
To override the `last_name` column of the `users` table:
1. Create a `users/` directory within this folder.
2. Create a `users/last_name.toml` file here strictly containing ONLY the values you wish to mutate.

```toml
[min_length]
value = 5
message = "Last name must be at least 5 characters long (Manual override!)"
```