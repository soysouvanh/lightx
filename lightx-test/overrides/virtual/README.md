#  Virtual Parameters (Payloads Without SQL)

Welcome to the **Virtual Parameters** directory!

**What is a "Virtual Parameter"?**
In LightX, most parameters coming in from an HTTP request are mapped directly to a database column (e.g. `users.first_name`). For those, LightX introspects your database and automatically builds their validation schema constraints.

However, your HTTP API will frequently require data that is **NOT** going into a database! For example:

- `accept_terms` (A UI checkbox indicating terms of service consent)
- `new_password` (A clear-text password that has no business being logged, to be encrypted later)
- `captcha_token` (A temporary antispam token for 3rd-party validation)

Because these elements do not exist in your SQL schema, LightX refers to them as **Virtual Parameters**.

##  How to create a Virtual Parameter?

To implement a new virtual parameter, simply drop a new `.toml` schema file inside this `virtual/` directory.

For instance, to enforce robust validation over an `accept_terms` checkbox, you would create `accept_terms.toml` here with the strict constraints required:

```toml
[type]
value = "bool"
message = "schema.type.message"

[is_optional]
value = false
message = "schema.is_optional.message" # Message to return if the payload is missing

#  PRO-TIP: "Enum Values" Blocks!
# If your parameter only allows a strict, small subset of words (e.g. booleans or static values), use `[enum_values]`.
# LightX will generate lightning-fast matching code (O(1) execution!) and will intelligently skip all heavy regex or length constraint generations!
[enum_values]
value = ["true", "false"]
message = "schema.enum_values.message"

[business_rules]
must_be_true = "overrides.virtual.accept_terms.business_rules.must_be_true"
```

##  Why define them here?

LightX's architecture focuses completely on **Performance (Zero-Overhead)** and **Reliability**.
Instead of manually writing conditional logic in your code to assert parameters (which could panic at runtime), LightX evaluates HTTP payloads using a **"Fail-Fast"** pipeline handled by its AOP (Aspect-Oriented Programming).
By defining your virtual parameter schema here, LightX incorporates it at build-time to shield your HTTP endpoints from fraudulent or malformed traffic natively!

##  How to use them in a Handler?

Once your `<name>.toml` file is in place here, open any route file located in `handlers/` (which builds your APIs), and map it by defining an **empty** SQL string targeting rule `""` !

```toml
# Inside handlers/MyHandler.toml for example
[parameters]
# The empty string "" tells the LightX Generator that this is a Virtual Parameter!
accept_terms = ""
```
