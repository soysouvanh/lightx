-- Dialect : SQLite

CREATE TABLE roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id INTEGER NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    FOREIGN KEY(role_id) REFERENCES roles(id)
);

CREATE INDEX idx_active_users ON users(is_active);

CREATE TABLE user_preferences (
    user_id INTEGER PRIMARY KEY,
    theme VARCHAR(50),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE user_groups (
    user_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(group_id) REFERENCES groups(id)
);

CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,
    name VARCHAR(100) NOT NULL,
    FOREIGN KEY(parent_id) REFERENCES categories(id)
);

CREATE TABLE all_types_demo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Strings
    string_col VARCHAR(255) NOT NULL,
    text_col TEXT,
    
    -- Numbers
    int_col INTEGER NOT NULL,
    bigint_col BIGINT,
    float_col REAL,
    double_col REAL,
    decimal_col DECIMAL(10, 2),
    
    -- Booleans
    bool_col BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Dates and Times
    date_col DATE,
    time_col TIME,
    datetime_col DATETIME,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Binary and others
    binary_col BLOB,
    json_col JSON
);
