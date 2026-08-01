-- Dialect : PostgreSQL

CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES roles(id),
    email VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_active_users ON users(is_active);

CREATE TABLE user_preferences (
    user_id INTEGER PRIMARY KEY REFERENCES users(id),
    theme VARCHAR(50)
);

CREATE TABLE groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE user_groups (
    user_id INTEGER NOT NULL REFERENCES users(id),
    group_id INTEGER NOT NULL REFERENCES groups(id),
    PRIMARY KEY (user_id, group_id)
);

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    parent_id INTEGER REFERENCES categories(id),
    name VARCHAR(100) NOT NULL
);

CREATE TABLE all_types_demo (
    id SERIAL PRIMARY KEY,
    
    -- Strings
    string_col VARCHAR(255) NOT NULL,
    text_col TEXT,
    
    -- Numbers
    int_col INTEGER NOT NULL,
    bigint_col BIGINT,
    float_col REAL,
    double_col DOUBLE PRECISION,
    decimal_col DECIMAL(10, 2),
    
    -- Booleans
    bool_col BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Dates and Times
    date_col DATE,
    time_col TIME,
    datetime_col TIMESTAMP,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Binary and others
    binary_col BYTEA,
    json_col JSONB
);
