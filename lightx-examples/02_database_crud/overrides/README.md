#  LightX Overrides Strategy

Welcome to the overrides directory!

The "Database-First" philosophy of LightX completely overwrites the `schema/` directory on each compilation.
**Never modify the files in `schema/` as they will be overwritten!**

If you want to override business validation rules (e.g., forcing `min_length = 5` on an SQL column), you must replicate the table's directory structure here.

## Example
To override the `last_name` column of the `users` table:
1. Create a `users/` directory here.
2. Create the file `users/last_name.toml` here with ONLY the values you wish to override.

```toml
[min_length]
value = 5
message = "The last name must be at least 5 characters long (Manual override!)"
```
