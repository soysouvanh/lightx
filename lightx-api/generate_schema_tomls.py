import os

def write_toml(path, rust_type, is_optional, min_length=0, max_length=None, is_primary_key=False, is_auto_increment=False, format_regex="^.*$"):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    
    # Calculate inferred lengths and defaults similarly to rust logic
    min_val, max_val = None, None
    if max_length is None:
        if rust_type == "i16":
            min_val, max_val, max_length = 0, 32767, 5
        elif rust_type == "i32":
            min_val, max_val, max_length = 0, 2147483647, 10
        elif rust_type == "i64":
            min_val, max_val, max_length = 0, 9223372036854775807, 19
            
    content = ""
    content += f"""[type]
value = "{rust_type}"
message = "schema.type.message"

"""
    
    opt_msg = '""' if is_optional else '"schema.is_optional.message"'
    default_str = 'default = "0"\n' if (not is_optional and rust_type in ["i32", "i64", "f64", "f32"]) else ('default = "false"\n' if not is_optional and rust_type == "bool" else "")
    
    content += f"""[is_optional]
value = {str(is_optional).lower()}
message = {opt_msg}
{default_str}
"""
    
    content += "# [enum_values]\n# value = []\n# message = \n\n"
    
    if max_length is not None:
        content += f"""[max_length]
value = {max_length}
message = "schema.max_length.message|{max_length}"

"""
        
    content += f"""[min_length]
value = {min_length}
message = "schema.min_length.message|{min_length}"

"""

    if min_val is not None:
        content += f"[min_value]\nvalue = {min_val}\nmessage = \"schema.min_value.message|{min_val}\"\n\n"
    else:
        content += "# [min_value]\n# value = \n# message = \n\n"
        
    if max_val is not None:
        content += f"[max_value]\nvalue = {max_val}\nmessage = \"schema.max_value.message|{max_val}\"\n\n"
    else:
        content += "# [max_value]\n# value = \n# message = \n\n"

    content += f"""[format]
value = '{format_regex}'
message = "schema.format.message"

[is_primary_key]
value = {str(is_primary_key).lower()}

[is_auto_increment]
value = {str(is_auto_increment).lower()}

[business_rules]
"""
    
    with open(path, "w") as f:
        f.write(content)

schemas = {
    "groups": {
        "id": ("i32", False, 1, None, True, True, "^[0-9]{1,10}$"),
        "name": ("String", False, 1, 100, False, False, "^.*$")
    },
    "user_groups": {
        "user_id": ("i32", False, 1, None, True, False, "^[0-9]{1,10}$"),
        "group_id": ("i32", False, 1, None, True, False, "^[0-9]{1,10}$")
    },
    "categories": {
        "id": ("i32", False, 1, None, True, True, "^[0-9]{1,10}$"),
        "parent_id": ("i32", True, 0, None, False, False, "^[0-9]{1,10}$"),
        "name": ("String", False, 1, 100, False, False, "^.*$")
    },
    "all_types_demo": {
        "id": ("i32", False, 1, None, True, True, "^[0-9]{1,10}$"),
        "string_col": ("String", False, 1, 255, False, False, "^.*$"),
        "text_col": ("String", True, 0, None, False, False, "^.*$"),
        "int_col": ("i32", False, 1, None, False, False, "^[0-9]{1,10}$"),
        "bigint_col": ("i64", True, 0, None, False, False, "^[0-9]{1,19}$"),
        "float_col": ("f32", True, 0, None, False, False, "^.*$"),
        "double_col": ("f64", True, 0, None, False, False, "^.*$"),
        "decimal_col": ("f64", True, 0, None, False, False, "^.*$"),
        "bool_col": ("bool", False, 1, None, False, False, "^[01]$"),
        "date_col": ("chrono::NaiveDate", True, 0, None, False, False, "^.*$"),
        "time_col": ("String", True, 0, None, False, False, "^.*$"),
        "datetime_col": ("chrono::DateTime<chrono::Utc>", True, 0, None, False, False, r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$"),
        "timestamp_col": ("chrono::DateTime<chrono::Utc>", True, 0, None, False, False, r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$"),
        "binary_col": ("Vec<u8>", True, 0, None, False, False, "^.*$"),
        "json_col": ("serde_json::Value", True, 0, None, False, False, "^.*$")
    }
}

base_dir = "/mnt/project/framework/lightx-workspace/lightx-api/schema"
databases = ["mysql", "postgres", "sqlite"]

for db in databases:
    for table, columns in schemas.items():
        table_path = os.path.join(base_dir, db, table)
        for col, (rust_type, is_optional, min_length, max_length, is_primary_key, is_auto_increment, format_regex) in columns.items():
            write_toml(
                os.path.join(table_path, f"{col}.toml"),
                rust_type, is_optional, min_length, max_length, is_primary_key, is_auto_increment, format_regex
            )

